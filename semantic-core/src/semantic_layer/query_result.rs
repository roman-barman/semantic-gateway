//! Converts DataFusion [`RecordBatch`]es into a JSON-serializable query result.

use crate::semantic_layer::query_result::column_meta::ColumnMeta;
use crate::semantic_layer::query_result::value_type::ValueType;
use datafusion::arrow::array::{
    Array, ArrayRef, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
    StringArray, StringViewArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use datafusion::arrow::compute::concat_batches;
use datafusion::arrow::datatypes::DataType;
use datafusion::arrow::record_batch::RecordBatch;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};

mod column_meta;
mod value_type;

/// The result of a successfully executed semantic query.
///
/// Columns are stored as an ordered `Vec` of `(name, array)` pairs rather than a map
/// to preserve schema ordering. Serializes to:
/// `{"schema": [...], "columns": {...}, "row_count": N}`.
#[derive(Debug)]
pub struct QueryResult {
    schema: Vec<ColumnMeta>,
    columns: Vec<(String, ArrayRef)>,
    row_count: usize,
}

#[cfg(test)]
impl QueryResult {
    pub fn row_count(&self) -> usize {
        self.row_count
    }
}

impl TryFrom<Vec<RecordBatch>> for QueryResult {
    type Error = QueryResultError;

    /// Converts a DataFusion result set into a `QueryResult`.
    ///
    /// DataFusion may partition a single query result across multiple [`RecordBatch`]es;
    /// this implementation concatenates them before column extraction.
    fn try_from(value: Vec<RecordBatch>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Self {
                schema: vec![],
                columns: vec![],
                row_count: 0,
            });
        }

        let schema = value
            .first()
            .ok_or(QueryResultError::Unexpected(
                "invalid record batch vector".to_string(),
            ))?
            .schema();
        let batch = concat_batches(&schema, &value).map_err(|_| QueryResultError::InvalidSchema)?;
        let (schema, arrays, row_count) = batch.into_parts();

        let schema_result = schema
            .fields()
            .iter()
            .map(|f| ColumnMeta::new(f.name().clone(), arrow_type_to_value_type(f.data_type())))
            .collect();

        let columns = schema
            .fields()
            .iter()
            .zip(arrays)
            .map(|(f, arr)| (f.name().clone(), arr))
            .collect();

        Ok(Self {
            schema: schema_result,
            columns,
            row_count,
        })
    }
}

fn arrow_type_to_value_type(dt: &DataType) -> ValueType {
    match dt {
        DataType::Utf8 | DataType::Utf8View => ValueType::String,
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => ValueType::Int,
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => ValueType::UInt,
        DataType::Float32 | DataType::Float64 => ValueType::Float,
        _ => ValueType::Unknown,
    }
}

macro_rules! serialize_primitive {
    ($seq:expr, $arr:expr, $arr_type:ty, $wide:ty) => {{
        let typed = $arr
            .as_any()
            .downcast_ref::<$arr_type>()
            .ok_or_else(|| serde::ser::Error::custom("invalid array type"))?;
        for i in 0..typed.len() {
            if typed.is_null(i) {
                $seq.serialize_element(&Option::<$wide>::None)?;
            } else {
                $seq.serialize_element(&(typed.value(i) as $wide))?;
            }
        }
    }};
}

macro_rules! serialize_string {
    ($seq:expr, $arr:expr, $arr_type:ty) => {{
        let typed = $arr
            .as_any()
            .downcast_ref::<$arr_type>()
            .ok_or_else(|| serde::ser::Error::custom("invalid array type"))?;
        for i in 0..typed.len() {
            if typed.is_null(i) {
                $seq.serialize_element(&Option::<&str>::None)?;
            } else {
                $seq.serialize_element(typed.value(i))?;
            }
        }
    }};
}

struct SerializableColumn<'a>(&'a dyn Array);

impl Serialize for SerializableColumn<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let arr = self.0;
        let mut seq = serializer.serialize_seq(Some(arr.len()))?;

        match arr.data_type() {
            DataType::Utf8 => serialize_string!(seq, arr, StringArray),
            DataType::Utf8View => serialize_string!(seq, arr, StringViewArray),
            DataType::Int8 => serialize_primitive!(seq, arr, Int8Array, i64),
            DataType::Int16 => serialize_primitive!(seq, arr, Int16Array, i64),
            DataType::Int32 => serialize_primitive!(seq, arr, Int32Array, i64),
            DataType::Int64 => serialize_primitive!(seq, arr, Int64Array, i64),
            DataType::UInt8 => serialize_primitive!(seq, arr, UInt8Array, u64),
            DataType::UInt16 => serialize_primitive!(seq, arr, UInt16Array, u64),
            DataType::UInt32 => serialize_primitive!(seq, arr, UInt32Array, u64),
            DataType::UInt64 => serialize_primitive!(seq, arr, UInt64Array, u64),
            DataType::Float32 => serialize_primitive!(seq, arr, Float32Array, f64),
            DataType::Float64 => serialize_primitive!(seq, arr, Float64Array, f64),
            _ => {
                for _ in 0..arr.len() {
                    seq.serialize_element(&Option::<()>::None)?;
                }
            }
        }

        seq.end()
    }
}

struct SerializableColumns<'a>(&'a [(String, ArrayRef)]);

impl Serialize for SerializableColumns<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (name, arr) in self.0 {
            map.serialize_entry(name, &SerializableColumn(arr.as_ref()))?;
        }
        map.end()
    }
}

impl Serialize for QueryResult {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("schema", &self.schema)?;
        map.serialize_entry("columns", &SerializableColumns(&self.columns))?;
        map.serialize_entry("row_count", &self.row_count)?;
        map.end()
    }
}

/// Errors that can occur while constructing a [`QueryResult`] from Arrow record batches.
#[derive(Debug, thiserror::Error)]
pub enum QueryResultError {
    #[error("unexpected error: {0}")]
    Unexpected(String),
    /// Arrow batch concatenation failed due to incompatible schemas across batches.
    #[error("invalid schema")]
    InvalidSchema,
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::arrow::array::{
        BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
        StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
    };
    use datafusion::arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    fn make_batch(schema: Arc<Schema>, arrays: Vec<ArrayRef>) -> RecordBatch {
        RecordBatch::try_new(schema, arrays).unwrap()
    }

    fn json(result: &QueryResult) -> serde_json::Value {
        serde_json::to_value(result).unwrap()
    }

    #[test]
    fn try_from_empty_vec_returns_empty_result() {
        // Arrange
        let batches: Vec<RecordBatch> = vec![];

        // Act
        let result = QueryResult::try_from(batches).unwrap();

        // Assert
        assert_eq!(result.row_count(), 0);
        let j = json(&result);
        assert_eq!(j["schema"], serde_json::json!([]));
        assert_eq!(j["columns"], serde_json::json!({}));
        assert_eq!(j["row_count"], 0);
    }

    #[test]
    fn try_from_single_batch_returns_correct_row_count() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));
        let batch = make_batch(schema, vec![Arc::new(Int64Array::from(vec![1i64, 2, 3]))]);

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        assert_eq!(result.row_count(), 3);
    }

    #[test]
    fn try_from_multiple_batches_concatenates_rows() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));
        let batch1 = make_batch(
            schema.clone(),
            vec![Arc::new(Int64Array::from(vec![1i64, 2]))],
        );
        let batch2 = make_batch(schema, vec![Arc::new(Int64Array::from(vec![3i64, 4, 5]))]);

        // Act
        let result = QueryResult::try_from(vec![batch1, batch2]).unwrap();

        // Assert
        assert_eq!(result.row_count(), 5);
    }

    #[test]
    fn schema_utf8_column_maps_to_string_value_type() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![Field::new("name", DataType::Utf8, false)]));
        let batch = make_batch(schema, vec![Arc::new(StringArray::from(vec!["a"]))]);

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        assert_eq!(json(&result)["schema"][0]["value_type"], "String");
    }

    #[test]
    fn schema_int_types_map_to_int_value_type() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![
            Field::new("i8", DataType::Int8, false),
            Field::new("i16", DataType::Int16, false),
            Field::new("i32", DataType::Int32, false),
            Field::new("i64", DataType::Int64, false),
        ]));
        let batch = make_batch(
            schema,
            vec![
                Arc::new(Int8Array::from(vec![1i8])),
                Arc::new(Int16Array::from(vec![1i16])),
                Arc::new(Int32Array::from(vec![1i32])),
                Arc::new(Int64Array::from(vec![1i64])),
            ],
        );

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        let schema_json = &json(&result)["schema"];
        for i in 0..4 {
            assert_eq!(schema_json[i]["value_type"], "Int");
        }
    }

    #[test]
    fn schema_uint_types_map_to_uint_value_type() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![
            Field::new("u8", DataType::UInt8, false),
            Field::new("u16", DataType::UInt16, false),
            Field::new("u32", DataType::UInt32, false),
            Field::new("u64", DataType::UInt64, false),
        ]));
        let batch = make_batch(
            schema,
            vec![
                Arc::new(UInt8Array::from(vec![1u8])),
                Arc::new(UInt16Array::from(vec![1u16])),
                Arc::new(UInt32Array::from(vec![1u32])),
                Arc::new(UInt64Array::from(vec![1u64])),
            ],
        );

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        let schema_json = &json(&result)["schema"];
        for i in 0..4 {
            assert_eq!(schema_json[i]["value_type"], "UInt");
        }
    }

    #[test]
    fn schema_float_types_map_to_float_value_type() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![
            Field::new("f32", DataType::Float32, false),
            Field::new("f64", DataType::Float64, false),
        ]));
        let batch = make_batch(
            schema,
            vec![
                Arc::new(Float32Array::from(vec![1.0f32])),
                Arc::new(Float64Array::from(vec![1.0f64])),
            ],
        );

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        let schema_json = &json(&result)["schema"];
        assert_eq!(schema_json[0]["value_type"], "Float");
        assert_eq!(schema_json[1]["value_type"], "Float");
    }

    #[test]
    fn schema_boolean_column_maps_to_unknown_value_type() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![Field::new(
            "flag",
            DataType::Boolean,
            false,
        )]));
        let batch = make_batch(schema, vec![Arc::new(BooleanArray::from(vec![true]))]);

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        assert_eq!(json(&result)["schema"][0]["value_type"], "Unknown");
    }

    #[test]
    fn serialize_int64_column_emits_i64_numbers() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![Field::new("val", DataType::Int64, false)]));
        let batch = make_batch(schema, vec![Arc::new(Int64Array::from(vec![10i64, 20]))]);

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        assert_eq!(json(&result)["columns"]["val"], serde_json::json!([10, 20]));
    }

    #[test]
    fn serialize_uint64_column_emits_u64_numbers() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![Field::new(
            "val",
            DataType::UInt64,
            false,
        )]));
        let batch = make_batch(schema, vec![Arc::new(UInt64Array::from(vec![5u64, 15]))]);

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        assert_eq!(json(&result)["columns"]["val"], serde_json::json!([5, 15]));
    }

    #[test]
    fn serialize_float64_column_emits_f64_numbers() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![Field::new(
            "val",
            DataType::Float64,
            false,
        )]));
        let batch = make_batch(
            schema,
            vec![Arc::new(Float64Array::from(vec![1.5f64, 2.5]))],
        );

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        assert_eq!(
            json(&result)["columns"]["val"],
            serde_json::json!([1.5, 2.5])
        );
    }

    #[test]
    fn serialize_float32_column_widens_to_f64() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![Field::new(
            "val",
            DataType::Float32,
            false,
        )]));
        let batch = make_batch(schema, vec![Arc::new(Float32Array::from(vec![3.0f32]))]);

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        assert_eq!(json(&result)["columns"]["val"], serde_json::json!([3.0]));
    }

    #[test]
    fn serialize_utf8_column_emits_json_strings() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![Field::new("city", DataType::Utf8, false)]));
        let batch = make_batch(
            schema,
            vec![Arc::new(StringArray::from(vec!["London", "Paris"]))],
        );

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        assert_eq!(
            json(&result)["columns"]["city"],
            serde_json::json!(["London", "Paris"])
        );
    }

    #[test]
    fn serialize_nullable_column_emits_null_for_missing_values() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![Field::new("val", DataType::Int64, true)]));
        let batch = make_batch(
            schema,
            vec![Arc::new(Int64Array::from(vec![Some(7i64), None, Some(9)]))],
        );

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        assert_eq!(
            json(&result)["columns"]["val"],
            serde_json::json!([7, null, 9])
        );
    }

    #[test]
    fn serialize_boolean_column_emits_all_null_values() {
        // Arrange
        let schema = Arc::new(Schema::new(vec![Field::new(
            "flag",
            DataType::Boolean,
            false,
        )]));
        let batch = make_batch(
            schema,
            vec![Arc::new(BooleanArray::from(vec![true, false]))],
        );

        // Act
        let result = QueryResult::try_from(vec![batch]).unwrap();

        // Assert
        assert_eq!(
            json(&result)["columns"]["flag"],
            serde_json::json!([null, null])
        );
    }

    #[test]
    fn serialize_empty_result_produces_correct_json_structure() {
        // Arrange
        let batches: Vec<RecordBatch> = vec![];

        // Act
        let result = QueryResult::try_from(batches).unwrap();

        // Assert
        assert_eq!(
            json(&result),
            serde_json::json!({"schema": [], "columns": {}, "row_count": 0})
        );
    }

    #[test]
    fn error_unexpected_display_includes_message() {
        // Arrange
        let err = QueryResultError::Unexpected("something went wrong".to_string());

        // Act
        let display = format!("{err}");

        // Assert
        assert_eq!(display, "unexpected error: something went wrong");
    }

    #[test]
    fn error_invalid_schema_display_is_correct() {
        // Arrange
        let err = QueryResultError::InvalidSchema;

        // Act
        let display = format!("{err}");

        // Assert
        assert_eq!(display, "invalid schema");
    }
}

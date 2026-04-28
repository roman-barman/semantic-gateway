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

#[derive(Debug)]
pub struct QueryResult {
    schema: Vec<ColumnMeta>,
    columns: Vec<(String, ArrayRef)>,
    row_count: usize,
}

impl QueryResult {
    pub fn row_count(&self) -> usize {
        self.row_count
    }
}

impl TryFrom<Vec<RecordBatch>> for QueryResult {
    type Error = QueryResultError;
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

struct SerializableColumn<'a>(&'a dyn Array);

impl Serialize for SerializableColumn<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let arr = self.0;
        let mut seq = serializer.serialize_seq(Some(arr.len()))?;

        macro_rules! serialize_primitive {
            ($arr_type:ty, $wide:ty) => {{
                let arr = arr
                    .as_any()
                    .downcast_ref::<$arr_type>()
                    .ok_or_else(|| serde::ser::Error::custom("invalid array type"))?;
                for i in 0..arr.len() {
                    if arr.is_null(i) {
                        seq.serialize_element(&Option::<$wide>::None)?;
                    } else {
                        seq.serialize_element(&(arr.value(i) as $wide))?;
                    }
                }
            }};
        }

        match arr.data_type() {
            DataType::Utf8 => {
                let arr = arr
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or_else(|| serde::ser::Error::custom("invalid array type"))?;
                for i in 0..arr.len() {
                    if arr.is_null(i) {
                        seq.serialize_element(&Option::<&str>::None)?;
                    } else {
                        seq.serialize_element(arr.value(i))?;
                    }
                }
            }
            DataType::Utf8View => {
                let arr = arr
                    .as_any()
                    .downcast_ref::<StringViewArray>()
                    .ok_or_else(|| serde::ser::Error::custom("invalid array type"))?;
                for i in 0..arr.len() {
                    if arr.is_null(i) {
                        seq.serialize_element(&Option::<&str>::None)?;
                    } else {
                        seq.serialize_element(arr.value(i))?;
                    }
                }
            }
            DataType::Int8 => serialize_primitive!(Int8Array, i64),
            DataType::Int16 => serialize_primitive!(Int16Array, i64),
            DataType::Int32 => serialize_primitive!(Int32Array, i64),
            DataType::Int64 => serialize_primitive!(Int64Array, i64),
            DataType::UInt8 => serialize_primitive!(UInt8Array, u64),
            DataType::UInt16 => serialize_primitive!(UInt16Array, u64),
            DataType::UInt32 => serialize_primitive!(UInt32Array, u64),
            DataType::UInt64 => serialize_primitive!(UInt64Array, u64),
            DataType::Float32 => serialize_primitive!(Float32Array, f64),
            DataType::Float64 => serialize_primitive!(Float64Array, f64),
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

#[derive(Debug, thiserror::Error)]
pub enum QueryResultError {
    #[error("unexpected error: {0}")]
    Unexpected(String),
    #[error("invalid schema")]
    InvalidSchema,
}

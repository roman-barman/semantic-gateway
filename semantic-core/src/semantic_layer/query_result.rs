use crate::semantic_layer::query_result::column_meta::ColumnMeta;
use crate::semantic_layer::query_result::column_value::ColumnValue;
use crate::semantic_layer::query_result::value_type::ValueType;
use datafusion::arrow::array::{
    Array, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array, StringArray,
    UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use datafusion::arrow::compute::concat_batches;
use datafusion::arrow::datatypes::DataType;
use datafusion::arrow::record_batch::RecordBatch;
use std::collections::HashMap;

mod column_meta;
mod column_value;
mod value_type;

#[derive(Debug, Clone, serde::Serialize)]
pub struct QueryResult {
    schema: Vec<ColumnMeta>,
    columns: HashMap<String, Vec<ColumnValue>>,
    row_count: usize,
}

impl TryFrom<Vec<RecordBatch>> for QueryResult {
    type Error = QueryResultError;
    fn try_from(value: Vec<RecordBatch>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Self {
                schema: vec![],
                columns: HashMap::new(),
                row_count: 0,
            });
        }

        let schema = value
            .first()
            .ok_or(QueryResultError::Unexpected("invalid vector".to_string()))?
            .schema();
        let batch = concat_batches(&schema, &value).map_err(|_| QueryResultError::InvalidSchema)?;
        let schema = batch.schema();
        let schema_result = schema
            .fields()
            .iter()
            .map(|field| {
                ColumnMeta::new(
                    field.name().to_string(),
                    arrow_type_to_value_type(field.data_type()),
                )
            })
            .collect();
        let mut result = HashMap::new();
        for col_idx in 0..batch.num_columns() {
            let name = schema.field(col_idx).name().to_string();
            let array = batch.column(col_idx);
            result.insert(name, serialize_column(array)?);
        }

        Ok(Self {
            schema: schema_result,
            columns: result,
            row_count: batch.num_rows(),
        })
    }
}

fn arrow_type_to_value_type(dt: &DataType) -> ValueType {
    match dt {
        DataType::Utf8 => ValueType::String,
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => ValueType::Int,
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => ValueType::UInt,
        DataType::Float32 | DataType::Float64 => ValueType::Float,
        _ => ValueType::Unknown,
    }
}

fn serialize_column(array: &dyn Array) -> Result<Vec<ColumnValue>, QueryResultError> {
    let len = array.len();

    macro_rules! cast_array {
        ($arr_type:ty, $variant:ident) => {{
            let arr =
                array
                    .as_any()
                    .downcast_ref::<$arr_type>()
                    .ok_or(QueryResultError::Unexpected(
                        "invalid array type".to_string(),
                    ))?;
            let result = (0..len)
                .map(|i| {
                    if arr.is_null(i) {
                        ColumnValue::Null
                    } else {
                        ColumnValue::$variant(arr.value(i).into())
                    }
                })
                .collect();
            Ok(result)
        }};
    }

    match array.data_type() {
        DataType::Utf8 => cast_array!(StringArray, String),
        DataType::Int8 => cast_array!(Int8Array, Int),
        DataType::Int16 => cast_array!(Int16Array, Int),
        DataType::Int32 => cast_array!(Int32Array, Int),
        DataType::Int64 => cast_array!(Int64Array, Int),
        DataType::UInt8 => cast_array!(UInt8Array, UInt),
        DataType::UInt16 => cast_array!(UInt16Array, UInt),
        DataType::UInt32 => cast_array!(UInt32Array, UInt),
        DataType::UInt64 => cast_array!(UInt64Array, UInt),
        DataType::Float32 => cast_array!(Float32Array, Float),
        DataType::Float64 => cast_array!(Float64Array, Float),
        _ => Ok(vec![ColumnValue::Null; len]),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueryResultError {
    #[error("Unexpected error: {0}")]
    Unexpected(String),
    #[error("Invalid schema")]
    InvalidSchema,
}

use crate::semantic_layer::query_result::value_type::ValueType;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ColumnMeta {
    name: String,
    value_type: ValueType,
}

impl ColumnMeta {
    pub fn new(name: String, value_type: ValueType) -> Self {
        Self { name, value_type }
    }
}

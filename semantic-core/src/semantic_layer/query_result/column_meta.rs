use crate::semantic_layer::query_result::value_type::ValueType;

/// Metadata for a single column in a [`QueryResult`](super::QueryResult).
///
/// Describes the column name and its semantic [`ValueType`] as they appear in the
/// `schema` array of the JSON response.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ColumnMeta {
    name: String,
    value_type: ValueType,
}

impl ColumnMeta {
    /// Creates a new `ColumnMeta` with the given column name and value type.
    pub fn new(name: String, value_type: ValueType) -> Self {
        Self { name, value_type }
    }
}

/// Semantic value type assigned to a result column, derived from the Arrow [`datafusion::arrow::datatypes::DataType`].
///
/// Arrow types are mapped to one of these variants during result construction.
/// Unsupported types (e.g., Boolean, Date) fall back to [`Unknown`](ValueType::Unknown).
#[derive(Debug, Clone, serde::Serialize)]
pub enum ValueType {
    String,
    Int,
    UInt,
    Float,
    /// Assigned for Arrow types with no semantic mapping; serialized as `null` for every row.
    Unknown,
}

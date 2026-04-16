#[derive(Debug, Clone, serde::Serialize)]
pub enum ValueType {
    String,
    Int,
    UInt,
    Float,
    Unknown,
}

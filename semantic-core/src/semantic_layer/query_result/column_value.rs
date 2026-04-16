#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
pub enum ColumnValue {
    String(String),
    Int(i64),
    UInt(u64),
    Float(f64),
    Null,
}

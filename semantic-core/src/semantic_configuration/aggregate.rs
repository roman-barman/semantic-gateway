#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Aggregate {
    Sum,
    Count,
}

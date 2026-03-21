#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Aggregate {
    Sum,
    Count,
}

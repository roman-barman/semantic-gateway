/// Aggregation function applied to a metric's source column.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Aggregate {
    /// Computes the sum of all non-null values.
    Sum,
    /// Counts non-null values.
    Count,
}

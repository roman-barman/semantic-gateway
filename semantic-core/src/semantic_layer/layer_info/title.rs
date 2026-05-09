/// Display label for a metric or dimension; used in model YAML, not exposed in query results.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub(super) struct Title(String);

#[cfg(test)]
impl Title {
    pub fn new(title: &str) -> Self {
        Title(title.to_string())
    }
}

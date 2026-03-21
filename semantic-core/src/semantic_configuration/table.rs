#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub(super) struct Table(String);

#[cfg(test)]
impl Table {
    pub fn new(table_name: &str) -> Self {
        Table(table_name.to_string())
    }
}

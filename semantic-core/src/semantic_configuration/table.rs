#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub(super) struct Table(String);

impl AsRef<str> for Table {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
impl Table {
    pub fn new(table_name: &str) -> Self {
        Table(table_name.to_string())
    }
}

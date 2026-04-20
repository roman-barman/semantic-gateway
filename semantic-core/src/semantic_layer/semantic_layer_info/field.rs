#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub(crate) struct Field(String);

impl AsRef<str> for Field {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
impl Field {
    pub fn new(field_name: &str) -> Self {
        Field(field_name.to_string())
    }
}

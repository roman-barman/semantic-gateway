#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub(super) struct Field(String);

#[cfg(test)]
impl Field {
    pub fn new(field_name: &str) -> Self {
        Field(field_name.to_string())
    }
}

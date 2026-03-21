use crate::semantic_configuration::field::Field;
use crate::semantic_configuration::title::Title;

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub(super) struct DimensionConfiguration {
    title: Title,
    field: Field,
}

#[cfg(test)]
impl DimensionConfiguration {
    pub fn new(title: &str, field: &str) -> Self {
        Self {
            title: Title::new(title),
            field: Field::new(field),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_configuration_deserialization() {
        let yaml = r#"
            title: "State"
            field: "state"
        "#;
        let dimension_config: DimensionConfiguration = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(dimension_config.title, Title::new("State"));
        assert_eq!(dimension_config.field, Field::new("state"));
    }
}

use crate::semantic_configuration::aggregate::Aggregate;
use crate::semantic_configuration::field::Field;
use crate::semantic_configuration::title::Title;

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub(super) struct MetricConfiguration {
    title: Title,
    aggregate: Aggregate,
    field: Field,
}

#[cfg(test)]
impl MetricConfiguration {
    pub fn new(title: &str, aggregate: Aggregate, field: &str) -> Self {
        Self {
            title: Title::new(title),
            aggregate,
            field: Field::new(field),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_configuration_deserialization() {
        let yaml = r#"
            title: "Revenue"
            aggregate: "sum"
            field: "amount"
        "#;

        let metric_config: MetricConfiguration = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(metric_config.title, Title::new("Revenue"));
        assert_eq!(metric_config.aggregate, Aggregate::Sum);
        assert_eq!(metric_config.field, Field::new("amount"));
    }
}

use crate::semantic_layer::layer_info::aggregate::Aggregate;
use crate::semantic_layer::layer_info::field::Field;
use crate::semantic_layer::layer_info::title::Title;

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub(crate) struct MetricConfiguration {
    title: Title,
    aggregate: Aggregate,
    field: Field,
}

impl MetricConfiguration {
    pub(crate) fn aggregate(&self) -> &Aggregate {
        &self.aggregate
    }

    pub(crate) fn field(&self) -> &Field {
        &self.field
    }
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

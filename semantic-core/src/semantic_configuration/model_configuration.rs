use crate::semantic_configuration::Field;
use crate::semantic_configuration::dimension_configuration::DimensionConfiguration;
use crate::semantic_configuration::metric_configuration::MetricConfiguration;
use crate::semantic_configuration::table::Table;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModelConfiguration {
    table: Table,
    metrics: HashMap<String, MetricConfiguration>,
    dimensions: HashMap<String, DimensionConfiguration>,
}

impl ModelConfiguration {
    pub(crate) fn table_name(&self) -> &str {
        &self.table.as_ref()
    }

    pub(crate) fn dimension_column(&self, dimension: &str) -> Option<&Field> {
        self.dimensions.get(dimension).map(|dim| dim.field())
    }

    pub(crate) fn get_metric_configuration(&self, metric: &str) -> Option<&MetricConfiguration> {
        self.metrics.get(metric)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_configuration::aggregate::Aggregate;

    #[test]
    fn test_model_configuration_deserialization() {
        let yaml = r#"
            table: "orders"
            metrics:
              revenue:
                title: "Revenue"
                aggregate: "sum"
                field: "amount"
              orders_count:
                title: "Orders count"
                aggregate: "count"
                field: "order_id"
            dimensions:
              country:
                title: "Country"
                field: "country"
              status:
                title: "Status"
                field: "status""#;

        let model_config: ModelConfiguration = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(model_config.table, Table::new("orders"));
        assert_eq!(model_config.metrics.len(), 2);
        assert_eq!(model_config.dimensions.len(), 2);

        let revenue = model_config.metrics.get("revenue").unwrap();
        assert_eq!(
            *revenue,
            MetricConfiguration::new("Revenue", Aggregate::Sum, "amount")
        );

        let orders_count = model_config.metrics.get("orders_count").unwrap();
        assert_eq!(
            *orders_count,
            MetricConfiguration::new("Orders count", Aggregate::Count, "order_id")
        );

        let country = model_config.dimensions.get("country").unwrap();
        assert_eq!(*country, DimensionConfiguration::new("Country", "country"));

        let status = model_config.dimensions.get("status").unwrap();
        assert_eq!(*status, DimensionConfiguration::new("Status", "status"));
    }
}

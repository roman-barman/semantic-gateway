use crate::semantic_layer::layer_info::dimension_configuration::DimensionConfiguration;
use crate::semantic_layer::layer_info::metric_configuration::MetricConfiguration;
use crate::semantic_layer::layer_info::table::Table;
use crate::{Dimension, Metric};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModelConfiguration {
    table: Table,
    metrics: HashMap<String, MetricConfiguration>,
    dimensions: HashMap<String, DimensionConfiguration>,
}

impl ModelConfiguration {
    pub(crate) fn table(&self) -> &Table {
        &self.table
    }

    pub(crate) fn dimension_config(
        &self,
        dimension: &Dimension,
    ) -> Option<&DimensionConfiguration> {
        self.dimensions.get(dimension.name())
    }

    pub(crate) fn metric_config(&self, metric: &Metric) -> Option<&MetricConfiguration> {
        self.metrics.get(metric.name())
    }

    pub(crate) fn column(&self, field: &str) -> Option<&str> {
        self.dimensions
            .get(field)
            .map(|dim| dim.field().as_ref())
            .or_else(|| {
                self.metrics
                    .get(field)
                    .map(|metric| metric.field().as_ref())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_layer::layer_info::aggregate::Aggregate;

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

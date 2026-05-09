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
impl ModelConfiguration {
    pub(crate) fn new(
        table: &str,
        metrics: HashMap<String, MetricConfiguration>,
        dimensions: HashMap<String, DimensionConfiguration>,
    ) -> Self {
        Self {
            table: Table::new(table),
            metrics,
            dimensions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_layer::layer_info::aggregate::Aggregate;

    fn make_model_config() -> ModelConfiguration {
        let metrics = HashMap::from([
            (
                "revenue".to_string(),
                MetricConfiguration::new("Revenue", Aggregate::Sum, "amount"),
            ),
            (
                "orders_count".to_string(),
                MetricConfiguration::new("Orders count", Aggregate::Count, "order_id"),
            ),
        ]);
        let dimensions = HashMap::from([
            (
                "country".to_string(),
                DimensionConfiguration::new("Country", "country"),
            ),
            (
                "status".to_string(),
                DimensionConfiguration::new("Status", "status"),
            ),
        ]);
        ModelConfiguration::new("orders", metrics, dimensions)
    }

    #[test]
    fn dimension_config_returns_config_for_existing_dimension() {
        // Arrange
        let model_config = make_model_config();

        // Act
        let result = model_config.dimension_config(&Dimension::new("country", "orders"));

        // Assert
        assert_eq!(
            result,
            Some(&DimensionConfiguration::new("Country", "country"))
        );
    }

    #[test]
    fn dimension_config_returns_none_for_unknown_dimension() {
        // Arrange
        let model_config = make_model_config();

        // Act
        let result = model_config.dimension_config(&Dimension::new("city", "orders"));

        // Assert
        assert_eq!(result, None);
    }

    #[test]
    fn metric_config_returns_config_for_existing_metric() {
        // Arrange
        let model_config = make_model_config();

        // Act
        let result = model_config.metric_config(&Metric::new("revenue", "orders"));

        // Assert
        assert_eq!(
            result,
            Some(&MetricConfiguration::new(
                "Revenue",
                Aggregate::Sum,
                "amount"
            ))
        );
    }

    #[test]
    fn metric_config_returns_none_for_unknown_metric() {
        // Arrange
        let model_config = make_model_config();

        // Act
        let result = model_config.metric_config(&Metric::new("profit", "orders"));

        // Assert
        assert_eq!(result, None);
    }

    #[test]
    fn column_returns_underlying_column_for_dimension() {
        // Arrange
        let model_config = make_model_config();

        // Act
        let result = model_config.column("country");

        // Assert
        assert_eq!(result, Some("country"));
    }

    #[test]
    fn column_returns_underlying_column_for_metric() {
        // Arrange
        let model_config = make_model_config();

        // Act
        let result = model_config.column("revenue");

        // Assert
        assert_eq!(result, Some("amount"));
    }

    #[test]
    fn column_returns_none_for_unknown_field() {
        // Arrange
        let model_config = make_model_config();

        // Act
        let result = model_config.column("profit");

        // Assert
        assert_eq!(result, None);
    }

    #[test]
    fn column_prefers_dimension_over_metric_on_name_collision() {
        // Arrange
        let model_config = ModelConfiguration::new(
            "orders",
            HashMap::from([(
                "price".to_string(),
                MetricConfiguration::new("Price metric", Aggregate::Sum, "price_met"),
            )]),
            HashMap::from([(
                "price".to_string(),
                DimensionConfiguration::new("Price dimension", "price_dim"),
            )]),
        );

        // Act
        let result = model_config.column("price");

        // Assert
        assert_eq!(result, Some("price_dim"));
    }

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

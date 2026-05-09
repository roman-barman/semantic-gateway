mod aggregate;
mod dimension_configuration;
mod field;
mod metric_configuration;
mod model_configuration;
mod table;
mod title;

pub(crate) use crate::semantic_layer::layer_info::aggregate::Aggregate;
use crate::semantic_layer::layer_info::dimension_configuration::DimensionConfiguration;
use crate::semantic_layer::layer_info::metric_configuration::MetricConfiguration;
pub use crate::semantic_layer::layer_info::model_configuration::ModelConfiguration;
use crate::semantic_layer::layer_info::table::Table;
use crate::{Dimension, Metric};
use std::collections::HashMap;

/// Holds the complete semantic layer configuration loaded at startup,
/// mapping model names to their [`ModelConfiguration`].
pub struct SemanticLayerInfo {
    layer: HashMap<String, ModelConfiguration>,
}

impl SemanticLayerInfo {
    /// Creates a new `SemanticLayerInfo` from a map of model name → configuration.
    pub fn new(layer: HashMap<String, ModelConfiguration>) -> Self {
        Self { layer }
    }

    /// Returns the table configuration for `model`, or `None` if the model is not registered.
    pub(crate) fn table(&self, model: &str) -> Option<&Table> {
        self.layer
            .get(model)
            .map(|model_config| model_config.table())
    }

    /// Returns the configuration for `dimension`, or `None` if the model or dimension is not registered.
    pub(crate) fn dimension_config(
        &self,
        dimension: &Dimension,
    ) -> Option<&DimensionConfiguration> {
        self.layer
            .get(dimension.model())
            .and_then(|model| model.dimension_config(dimension))
    }

    /// Returns the configuration for `metric`, or `None` if the model or metric is not registered.
    pub(crate) fn metric_config(&self, metric: &Metric) -> Option<&MetricConfiguration> {
        self.layer
            .get(metric.model())
            .and_then(|model| model.metric_config(metric))
    }

    /// Returns the source column name for a semantic `field` in `model`, or `None` if not found.
    pub(crate) fn column(&self, model: &str, field: &str) -> Option<&str> {
        self.layer
            .get(model)
            .and_then(|model_config| model_config.column(field))
    }
}

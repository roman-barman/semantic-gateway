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

pub struct SemanticLayerInfo {
    layer: HashMap<String, ModelConfiguration>,
}

impl SemanticLayerInfo {
    pub fn new(layer: HashMap<String, ModelConfiguration>) -> Self {
        Self { layer }
    }

    pub(crate) fn table(&self, model: &str) -> Option<&Table> {
        self.layer
            .get(model)
            .map(|model_config| model_config.table())
    }

    pub(crate) fn dimension_config(
        &self,
        dimension: &Dimension,
    ) -> Option<&DimensionConfiguration> {
        self.layer
            .get(dimension.model())
            .and_then(|model| model.dimension_config(dimension))
    }

    pub(crate) fn metric_config(&self, metric: &Metric) -> Option<&MetricConfiguration> {
        self.layer
            .get(metric.model())
            .and_then(|model| model.metric_config(metric))
    }
}

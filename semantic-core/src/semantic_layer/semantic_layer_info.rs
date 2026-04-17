use crate::ModelConfiguration;
use crate::semantic_configuration::{Aggregate, Field};
use std::collections::HashMap;

pub struct SemanticLayerInfo {
    layer: HashMap<String, ModelConfiguration>,
}

impl SemanticLayerInfo {
    pub fn new(layer: HashMap<String, ModelConfiguration>) -> Self {
        Self { layer }
    }

    pub(crate) fn table(&self, model: &str) -> Option<&str> {
        self.layer
            .get(model)
            .map(|model_config| model_config.table_name())
    }

    pub(crate) fn get_dimension_column(&self, model: &str, dimension: &str) -> Option<&Field> {
        self.layer
            .get(model)
            .and_then(|model| model.dimension_column(dimension))
    }

    pub(crate) fn get_metric_info(
        &self,
        table: &str,
        metric: &str,
    ) -> Option<(&Aggregate, &Field)> {
        self.layer
            .get(table)
            .and_then(|model| model.get_metric_configuration(metric))
            .map(|metric_config| (metric_config.aggregate(), metric_config.field()))
    }
}

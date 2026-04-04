use crate::ModelConfiguration;
use crate::semantic_configuration::{Aggregate, Field};
use std::collections::HashMap;

pub(super) struct SemanticLayerInfo {
    layer: HashMap<String, ModelConfiguration>,
}

impl SemanticLayerInfo {
    fn new(models: Vec<ModelConfiguration>) -> Self {
        let layer = models
            .into_iter()
            .map(|model| (model.table_name().to_string(), model))
            .collect();

        Self { layer }
    }

    pub(crate) fn get_dimension_column(&self, table: &str, dimension: &str) -> Option<&Field> {
        self.layer
            .get(table)
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
            .and_then(|metric_config| Some((metric_config.aggregate(), metric_config.field())))
    }
}

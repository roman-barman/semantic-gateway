use crate::ModelConfiguration;
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
}

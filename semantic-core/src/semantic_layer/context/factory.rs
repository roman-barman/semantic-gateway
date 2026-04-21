use crate::data_source::DataSource;
use crate::{SemanticLayerContext, SemanticLayerInfo};
use datafusion::execution::runtime_env::RuntimeEnv;
use datafusion::prelude::{SessionConfig, SessionContext};
use std::sync::Arc;

pub struct SemanticLayerContextFactory {
    layer_info: Arc<SemanticLayerInfo>,
    data_source: Arc<dyn DataSource>,
    runtime: Arc<RuntimeEnv>,
    config: SessionConfig,
}

impl SemanticLayerContextFactory {
    pub fn new(layer_info: Arc<SemanticLayerInfo>, data_source: Arc<dyn DataSource>) -> Self {
        Self {
            layer_info,
            data_source,
            runtime: Arc::new(RuntimeEnv::default()),
            config: SessionConfig::default(),
        }
    }

    pub fn create(&self) -> SemanticLayerContext {
        let session_context =
            SessionContext::new_with_config_rt(self.config.clone(), self.runtime.clone());
        SemanticLayerContext::new_with_context(
            session_context,
            self.layer_info.clone(),
            self.data_source.clone(),
        )
    }
}

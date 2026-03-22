use crate::semantic_layer::semantic_layer_info::SemanticLayerInfo;
use datafusion::prelude::SessionContext;
use std::rc::Rc;

struct SemanticLayerContext {
    semantic_layer_info: Rc<SemanticLayerInfo>,
    context: SessionContext,
}

impl SemanticLayerContext {
    fn new(semantic_layer_info: Rc<SemanticLayerInfo>) -> Self {
        SemanticLayerContext {
            semantic_layer_info,
            context: SessionContext::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum ExecutionQueryError {}

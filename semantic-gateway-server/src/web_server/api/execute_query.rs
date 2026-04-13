use actix_web::{HttpResponse, post, web};
use semantic_core::{SemanticLayerContext, SemanticLayerInfo};

#[post("/query/execute")]
#[tracing::instrument(name = "Execute query", skip(semantic_layer_info))]
pub(crate) async fn execute_query(
    semantic_layer_info: web::Data<SemanticLayerInfo>,
    request: web::Json<QueryRequest>,
) -> HttpResponse {
    let context = SemanticLayerContext::new(&semantic_layer_info);
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize, Debug)]
struct QueryRequest {
    metrics: Vec<String>,
    dimensions: Vec<String>,
}

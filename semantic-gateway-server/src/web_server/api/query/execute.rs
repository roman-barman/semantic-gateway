use crate::web_server::api::query::{QueryRequest, map_to_query};
use crate::web_server::error::ServerError;
use actix_web::{HttpResponse, post, web};
use semantic_core::{ExecutionQueryError, SemanticLayerContextFactory};

#[post("/query/execute")]
#[tracing::instrument(name = "Execute query", skip(context_factory))]
pub(crate) async fn execute_query(
    context_factory: web::Data<SemanticLayerContextFactory>,
    request: web::Json<QueryRequest>,
) -> Result<HttpResponse, ServerError> {
    let context = context_factory.create();
    let query = map_to_query(&request)?;
    let result = context.execute_query(&query).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<ExecutionQueryError> for ServerError {
    fn from(err: ExecutionQueryError) -> Self {
        ServerError::InternalServerError(err.to_string())
    }
}

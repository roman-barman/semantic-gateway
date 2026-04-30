use crate::web_server::error::ServerError;
use actix_web::{HttpResponse, post, web};
use semantic_core::query::Query;
use semantic_core::{Dimension, ExecutionQueryError, Metric, SemanticLayerContextFactory};

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

#[derive(serde::Deserialize, Debug)]
struct QueryRequest {
    metrics: Vec<String>,
    dimensions: Vec<String>,
}

fn map_to_query(request: &QueryRequest) -> Result<Query<'_>, QueryError> {
    let metrics: Vec<Metric> = request
        .metrics
        .iter()
        .map(|s| (split_reference(s), s))
        .map(|(parts, original)| match parts {
            None => Err(QueryError::InvalidMetric(original.clone())),
            Some((model, name)) => {
                if model.is_empty() || name.is_empty() {
                    Err(QueryError::InvalidMetric(original.clone()))
                } else {
                    Ok(Metric::new(name, model))
                }
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let dimensions: Vec<Dimension> = request
        .dimensions
        .iter()
        .map(|s| (split_reference(s), s))
        .map(|(parts, original)| match parts {
            None => Err(QueryError::InvalidDimension(original.clone())),
            Some((model, name)) => {
                if model.is_empty() || name.is_empty() {
                    Err(QueryError::InvalidDimension(original.clone()))
                } else {
                    Ok(Dimension::new(name, model))
                }
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Query::new(metrics, dimensions, vec![]))
}

fn split_reference(value: &str) -> Option<(&str, &str)> {
    value.split_once('.')
}

#[derive(thiserror::Error, Debug, Clone)]
enum QueryError {
    #[error("invalid metric: {0}")]
    InvalidMetric(String),
    #[error("invalid dimension: {0}")]
    InvalidDimension(String),
}

impl From<QueryError> for ServerError {
    fn from(err: QueryError) -> Self {
        ServerError::UnprocessableEntity(err.to_string())
    }
}

impl From<ExecutionQueryError> for ServerError {
    fn from(err: ExecutionQueryError) -> Self {
        ServerError::InternalServerError(err.to_string())
    }
}

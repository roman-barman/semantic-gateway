use actix_web::{HttpResponse, post, web};
use semantic_core::data_source::DataSource;
use semantic_core::query::{Dimension, Metric, Query};
use semantic_core::{SemanticLayerContext, SemanticLayerInfo};

#[post("/query/execute")]
#[tracing::instrument(name = "Execute query", skip(semantic_layer_info, data_source))]
pub(crate) async fn execute_query(
    semantic_layer_info: web::Data<SemanticLayerInfo>,
    data_source: web::Data<dyn DataSource>,
    request: web::Json<QueryRequest>,
) -> HttpResponse {
    let context = SemanticLayerContext::new(&semantic_layer_info, data_source.as_ref());
    let query = map_to_query(&request);
    match query {
        Err(err) => {
            tracing::error!("Failed to map request to query: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
        Ok(query) => {
            let result = context.execute_query(&query).await;
            match result {
                Err(err) => {
                    tracing::error!("Failed to execute query: {:?}", err);
                    HttpResponse::InternalServerError().finish()
                }
                Ok(result) => HttpResponse::Ok().json(result),
            }
        }
    }
}

#[derive(serde::Deserialize, Debug)]
struct QueryRequest {
    metrics: Vec<String>,
    dimensions: Vec<String>,
}

fn map_to_query(request: &QueryRequest) -> Result<Query<'_>, Vec<QueryError>> {
    let metrics: Vec<Result<Metric, QueryError>> = request
        .metrics
        .iter()
        .map(|s| split_reference(s))
        .map(|parts| {
            if parts.len() != 2 {
                return Err(QueryError::InvalidMetric(parts.join(".")));
            }
            Ok(Metric::new(parts[1], parts[0]))
        })
        .collect();

    let dimensions: Vec<Result<Dimension, QueryError>> = request
        .dimensions
        .iter()
        .map(|s| split_reference(s))
        .map(|parts| {
            if parts.len() != 2 {
                return Err(QueryError::InvalidDimension(parts.join(".")));
            }
            Ok(Dimension::new(parts[1], parts[0]))
        })
        .collect();

    let mut errors: Vec<QueryError> = metrics
        .iter()
        .filter_map(|r| r.as_ref().err().cloned())
        .collect();
    errors.extend(dimensions.iter().filter_map(|r| r.as_ref().err().cloned()));

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(Query::new(
        metrics.into_iter().filter_map(|r| r.ok()).collect(),
        dimensions.into_iter().filter_map(|r| r.ok()).collect(),
    ))
}

fn split_reference(value: &str) -> Vec<&str> {
    value.split('.').collect()
}

#[derive(thiserror::Error, Debug, Clone)]
enum QueryError {
    #[error("Invalid metric: {0}")]
    InvalidMetric(String),
    #[error("Invalid dimension: {0}")]
    InvalidDimension(String),
}

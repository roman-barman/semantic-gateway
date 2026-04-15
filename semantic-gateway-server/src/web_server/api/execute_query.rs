use actix_web::{HttpResponse, post, web};
use semantic_core::query::{Dimension, Metric, Query};
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

fn map_to_query(request: &QueryRequest) -> Result<Query, Vec<QueryError>> {
    let metrics: Vec<Result<Metric, QueryError>> = request
        .metrics
        .iter()
        .map(split_to_parts)
        .map(|parts| {
            if parts.len() != 2 {
                return Err(QueryError::InvalidMeasure(parts.join(".")));
            }
            Ok(Metric::new(parts[0], parts[1]))
        })
        .collect();

    let dimensions: Vec<Result<Dimension, QueryError>> = request
        .dimensions
        .iter()
        .map(split_to_parts)
        .map(|parts| {
            if parts.len() != 2 {
                return Err(QueryError::InvalidDimension(parts.join(".")));
            }
            Ok(Dimension::new(parts[0], parts[1]))
        })
        .collect();

    let mut errors: Vec<QueryError> = metrics
        .iter()
        .filter(|metric| metric.is_err())
        .map(|metric| metric.as_ref().err().cloned())
        .flatten()
        .collect();
    errors.append(
        &mut dimensions
            .iter()
            .filter(|dimension| dimension.is_err())
            .map(|dimension| dimension.as_ref().err().cloned())
            .flatten()
            .collect(),
    );

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(Query::new(
        metrics
            .into_iter()
            .filter_map(|metric| metric.ok())
            .collect(),
        dimensions
            .into_iter()
            .filter_map(|dimension| dimension.ok())
            .collect(),
    ))
}

fn split_to_parts(value: &String) -> Vec<&str> {
    value.split('.').collect()
}

#[derive(thiserror::Error, Debug, Clone)]
enum QueryError {
    #[error("Invalid measure: {0}")]
    InvalidMeasure(String),
    #[error("Invalid dimension: {0}")]
    InvalidDimension(String),
}

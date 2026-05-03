use crate::web_server::error::ServerError;
use semantic_core::query::Query;
use semantic_core::{Dimension, Metric};

mod execute;

pub(crate) use execute::*;

#[derive(serde::Deserialize, Debug)]
pub(crate) struct QueryRequest {
    metrics: Vec<String>,
    dimensions: Vec<String>,
    filters: Vec<Filter>,
}

#[derive(serde::Deserialize, Debug)]
pub(crate) struct Filter {
    filed: String,
    op: String,
    value: Primitive,
}
#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum Primitive {
    Int(i64),
    Float(f64),
    String(String),
}

pub(crate) fn map_to_query(request: &QueryRequest) -> Result<Query<'_>, QueryError> {
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
pub(crate) enum QueryError {
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

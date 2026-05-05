use crate::web_server::error::ServerError;
use semantic_core::query::Query;
use semantic_core::{Dimension, Metric};

mod execute;

pub(crate) use execute::*;

#[derive(serde::Deserialize, Debug)]
pub(crate) struct QueryRequest {
    metrics: Vec<String>,
    dimensions: Vec<String>,
    filters: Option<Vec<Filter>>,
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

impl<'a> TryFrom<&'a QueryRequest> for Query<'a> {
    type Error = QueryError;

    fn try_from(value: &'a QueryRequest) -> Result<Self, Self::Error> {
        let metrics: Vec<Metric> = value
            .metrics
            .iter()
            .map(|s| (split_reference(s), s))
            .map(|(parts, original)| match parts {
                None => Err(QueryError::Metric(original.clone())),
                Some((model, name)) => {
                    if model.is_empty() || name.is_empty() {
                        Err(QueryError::Metric(original.clone()))
                    } else {
                        Ok(Metric::new(name, model))
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let dimensions: Vec<Dimension> = value
            .dimensions
            .iter()
            .map(|s| (split_reference(s), s))
            .map(|(parts, original)| match parts {
                None => Err(QueryError::Dimension(original.clone())),
                Some((model, name)) => {
                    if model.is_empty() || name.is_empty() {
                        Err(QueryError::Dimension(original.clone()))
                    } else {
                        Ok(Dimension::new(name, model))
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let filters: Vec<semantic_core::Filter<'_>> = match &value.filters {
            Some(filters) => filters
                .iter()
                .map(|f| f.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            None => vec![],
        };

        Ok(Query::new(metrics, dimensions, filters))
    }
}

impl<'a> TryFrom<&'a Filter> for semantic_core::Filter<'a> {
    type Error = QueryError;
    fn try_from(value: &'a Filter) -> Result<Self, Self::Error> {
        let (model, filed) = split_reference(value.filed.as_str())
            .ok_or(QueryError::FilterFieldName(value.filed.clone()))?;

        if model.is_empty() || filed.is_empty() {
            return Err(QueryError::FilterFieldName(value.filed.clone()));
        }

        let operation = match value.op.as_str() {
            "eq" => semantic_core::FilterOperation::Eq,
            "lt" => semantic_core::FilterOperation::Lt,
            "gt" => semantic_core::FilterOperation::Gt,
            "lte" => semantic_core::FilterOperation::Lte,
            "gte" => semantic_core::FilterOperation::Gte,
            "ne" => semantic_core::FilterOperation::Ne,
            _ => return Err(QueryError::FilterOperation(value.op.clone())),
        };

        let filter_value = match &value.value {
            Primitive::Float(number) => semantic_core::FilterValue::Float(*number),
            Primitive::String(string) => semantic_core::FilterValue::String(string.as_str()),
            Primitive::Int(number) => semantic_core::FilterValue::Int(*number),
        };

        Ok(semantic_core::Filter::new(
            filed,
            model,
            operation,
            filter_value,
        ))
    }
}

fn split_reference(value: &str) -> Option<(&str, &str)> {
    value.split_once('.')
}

#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum QueryError {
    #[error("invalid metric: {0}")]
    Metric(String),
    #[error("invalid dimension: {0}")]
    Dimension(String),
    #[error("invalid filter field name: {0}")]
    FilterFieldName(String),
    #[error("invalid filter operation: {0}")]
    FilterOperation(String),
}

impl From<QueryError> for ServerError {
    fn from(err: QueryError) -> Self {
        ServerError::UnprocessableEntity(err.to_string())
    }
}

use crate::semantic_layer::filter::Filter;
use crate::{Dimension, Metric};
use std::collections::HashSet;

#[derive(Debug, Clone, thiserror::Error)]
pub enum QueryValidationError {
    #[error("duplicate metric: model '{model}', name '{name}'")]
    DuplicateMetric { model: String, name: String },
    #[error("duplicate dimension: model '{model}', name '{name}'")]
    DuplicateDimension { model: String, name: String },
    #[error("duplicate filter")]
    DuplicateFilter,
}

pub struct Query<'a> {
    metrics: Vec<Metric<'a>>,
    dimensions: Vec<Dimension<'a>>,
    filters: Vec<Filter<'a>>,
}

impl<'a> Query<'a> {
    pub fn try_new(
        metrics: Vec<Metric<'a>>,
        dimensions: Vec<Dimension<'a>>,
        filters: Vec<Filter<'a>>,
    ) -> Result<Self, QueryValidationError> {
        let mut seen = HashSet::new();
        for m in &metrics {
            if !seen.insert(m) {
                return Err(QueryValidationError::DuplicateMetric {
                    model: m.model().to_owned(),
                    name: m.name().to_owned(),
                });
            }
        }

        let mut seen = HashSet::new();
        for d in &dimensions {
            if !seen.insert(d) {
                return Err(QueryValidationError::DuplicateDimension {
                    model: d.model().to_owned(),
                    name: d.name().to_owned(),
                });
            }
        }

        for (i, a) in filters.iter().enumerate() {
            if filters[i + 1..].iter().any(|b| a == b) {
                return Err(QueryValidationError::DuplicateFilter);
            }
        }

        Ok(Self {
            metrics,
            dimensions,
            filters,
        })
    }

    pub(super) fn metrics(&self) -> &[Metric<'_>] {
        &self.metrics
    }

    pub(super) fn dimensions(&self) -> &[Dimension<'_>] {
        &self.dimensions
    }

    pub(super) fn filters(&self) -> &[Filter<'_>] {
        &self.filters
    }

    pub(super) fn models(&self) -> HashSet<&str> {
        let metrics_models: HashSet<&str> =
            self.metrics.iter().map(|metric| metric.model()).collect();
        let dimension_models: HashSet<&str> = self
            .dimensions
            .iter()
            .map(|dimension| dimension.model())
            .collect();
        metrics_models.union(&dimension_models).cloned().collect()
    }
}

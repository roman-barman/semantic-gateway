use crate::semantic_layer::filter::Filter;
use crate::{Dimension, Metric};
use std::collections::HashSet;

pub struct Query<'a> {
    metrics: Vec<Metric<'a>>,
    dimensions: Vec<Dimension<'a>>,
    filters: Vec<Filter<'a>>,
}

impl<'a> Query<'a> {
    pub fn new(
        metrics: Vec<Metric<'a>>,
        dimensions: Vec<Dimension<'a>>,
        filters: Vec<Filter<'a>>,
    ) -> Self {
        Self {
            metrics,
            dimensions,
            filters,
        }
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

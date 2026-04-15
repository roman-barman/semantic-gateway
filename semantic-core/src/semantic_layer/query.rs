pub use crate::semantic_layer::query::dimension::Dimension;
pub use crate::semantic_layer::query::metric::Metric;
use std::collections::HashSet;

mod dimension;
mod metric;

pub struct Query<'a> {
    metrics: Vec<Metric<'a>>,
    dimensions: Vec<Dimension<'a>>,
}

impl<'a> Query<'a> {
    pub fn new(metrics: Vec<Metric<'a>>, dimensions: Vec<Dimension<'a>>) -> Self {
        Self {
            metrics,
            dimensions,
        }
    }

    pub(super) fn metrics(&self) -> &[Metric] {
        &self.metrics
    }

    pub(super) fn dimensions(&self) -> &[Dimension] {
        &self.dimensions
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

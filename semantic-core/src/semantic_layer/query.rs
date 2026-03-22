use crate::semantic_layer::query::dimension::Dimension;
use crate::semantic_layer::query::metric::Metric;
use std::collections::HashSet;

mod dimension;
mod metric;

pub(super) struct Query {
    metrics: Vec<Metric>,
    dimensions: Vec<Dimension>,
}

impl Query {
    pub(super) fn metrics(&self) -> &Vec<Metric> {
        &self.metrics
    }

    pub(super) fn dimensions(&self) -> &Vec<Dimension> {
        &self.dimensions
    }

    pub(super) fn tables(&self) -> HashSet<&str> {
        let metrics_tables: HashSet<&str> = self
            .metrics
            .iter()
            .map(|metric| metric.table_name())
            .collect();
        let dimension_tables: HashSet<&str> = self
            .dimensions
            .iter()
            .map(|dimension| dimension.table_name())
            .collect();
        metrics_tables.union(&dimension_tables).cloned().collect()
    }
}

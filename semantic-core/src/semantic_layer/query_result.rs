use crate::semantic_layer::query_result::result_row::ResultRow;

mod column;
mod result_row;
mod result_value;

pub struct QueryResult {
    rows: Vec<ResultRow>,
}

impl QueryResult {
    pub(crate) fn new(rows: Vec<ResultRow>) -> Self {
        Self { rows }
    }

    pub(crate) fn empty() -> Self {
        Self { rows: Vec::new() }
    }
}

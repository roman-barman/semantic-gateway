use crate::semantic_layer::query_result::result_row::ResultRow;

mod column;
mod result_row;
mod result_value;

pub(super) struct QueryResult {
    rows: Vec<ResultRow>,
}

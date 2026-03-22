use crate::semantic_layer::query_result::column::Column;
use crate::semantic_layer::query_result::result_value::ResultValue;

pub(super) struct ResultRow {
    row: Vec<(Column, ResultValue)>,
}

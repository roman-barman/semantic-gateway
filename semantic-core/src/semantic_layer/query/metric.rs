pub(super) struct Metric {
    field: String,
    table: String,
}

impl Metric {
    pub fn table_name(&self) -> &str {
        &self.table
    }
}

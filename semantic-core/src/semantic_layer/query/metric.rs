pub(crate) struct Metric {
    field: String,
    table: String,
}

impl Metric {
    pub fn table_name(&self) -> &str {
        &self.table
    }

    pub fn field_name(&self) -> &str {
        &self.field
    }
}

pub(crate) struct Dimension {
    field: String,
    table: String,
}

impl Dimension {
    pub(crate) fn table_name(&self) -> &str {
        &self.table
    }
    pub(crate) fn field_name(&self) -> &str {
        &self.field
    }
}

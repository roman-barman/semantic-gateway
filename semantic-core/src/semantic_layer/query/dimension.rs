pub(super) struct Dimension {
    field: String,
    table: String,
}

impl Dimension {
    pub fn table_name(&self) -> &str {
        &self.table
    }
}

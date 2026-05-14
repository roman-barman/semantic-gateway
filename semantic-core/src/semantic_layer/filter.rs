#[derive(PartialEq)]
pub struct Filter<'a> {
    field: &'a str,
    model: &'a str,
    operation: FilterOperation,
    value: FilterValue<'a>,
}

impl<'a> Filter<'a> {
    pub fn new(
        field: &'a str,
        model: &'a str,
        operation: FilterOperation,
        value: FilterValue<'a>,
    ) -> Self {
        Filter {
            field,
            model,
            operation,
            value,
        }
    }

    pub(crate) fn field(&self) -> &'a str {
        self.field
    }

    pub(crate) fn model(&self) -> &'a str {
        self.model
    }

    pub(crate) fn operation(&self) -> &FilterOperation {
        &self.operation
    }

    pub(crate) fn value(&self) -> &FilterValue<'a> {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterOperation {
    Eq,
    Lt,
    Gt,
    Lte,
    Gte,
    Ne,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue<'a> {
    String(&'a str),
    Int(i64),
    Float(f64),
}

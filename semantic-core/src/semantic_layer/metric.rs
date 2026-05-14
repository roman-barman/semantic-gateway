#[derive(PartialEq, Eq, Hash)]
pub struct Metric<'a> {
    name: &'a str,
    model: &'a str,
}

impl<'a> Metric<'a> {
    pub fn new(name: &'a str, model: &'a str) -> Self {
        Metric { name, model }
    }

    pub(crate) fn model(&self) -> &str {
        self.model
    }

    pub(crate) fn name(&self) -> &str {
        self.name
    }
}

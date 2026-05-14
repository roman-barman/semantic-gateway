#[derive(PartialEq, Eq, Hash)]
pub struct Dimension<'a> {
    name: &'a str,
    model: &'a str,
}

impl<'a> Dimension<'a> {
    pub fn new(name: &'a str, model: &'a str) -> Self {
        Dimension { name, model }
    }

    pub(crate) fn model(&self) -> &str {
        self.model
    }

    pub(crate) fn name(&self) -> &str {
        self.name
    }
}

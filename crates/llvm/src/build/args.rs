#[derive(Debug, PartialEq)]
pub enum BuildTarget {
    ByName(String),
    ByIndex(usize),
}

impl Default for BuildTarget {
    fn default() -> Self {
        BuildTarget::ByIndex(1)
    }
}

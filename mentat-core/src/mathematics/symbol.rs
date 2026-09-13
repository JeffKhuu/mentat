#[derive(Debug, PartialEq, Clone)]
pub struct Symbol(String);

impl Symbol {
    pub fn create(s: String) -> Symbol {
        Symbol(s)
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self {
        Symbol::create(value.to_string())
    }
}

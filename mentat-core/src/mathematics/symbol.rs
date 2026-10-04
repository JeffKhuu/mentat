use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Eq, Ord, PartialOrd, Hash)]
pub struct Symbol(String);

impl Symbol {
    pub fn create(s: String) -> Symbol {
        Symbol(s)
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0}", self.0)
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self {
        Symbol::create(value.to_string())
    }
}

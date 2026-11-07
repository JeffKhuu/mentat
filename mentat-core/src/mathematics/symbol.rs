use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Eq, Ord, PartialOrd, Hash)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(s: String) -> Identifier {
        Identifier(s)
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0}", self.0)
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Identifier::new(value.to_string())
    }
}

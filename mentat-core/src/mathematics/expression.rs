use crate::{mathematics::symbol::Symbol, parser::numerical_string::NumericalString};

/**
Mentat's core expression type

Examples:

2 + 2
x^2 + 2
f(x)
*/
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    // Atomic Expressions
    Integer(u64),
    Real(f64),

    // Binary Expressions
    Add(Vec<Expr>),
    Mul(Vec<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    // We define division (x / y) as x * y^(-1)
    // We define subtraction (a - b) as a + (-b)

    // Unary Expressions
    Neg(Box<Expr>),

    // Functional Expression
    Call { function: Symbol, args: Vec<Expr> },
}

impl From<&NumericalString> for Expr {
    fn from(value: &NumericalString) -> Self {
        if value.is_integer() {
            // We can be confident we have parsed an integer because of the above guard
            return Expr::Integer(value.to_integer().unwrap());
        }
        Expr::Real(value.to_float().unwrap())
    }
}

impl From<&str> for Expr {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl From<String> for Expr {
    fn from(value: String) -> Self {
        if let Some(s) = NumericalString::create(value.as_str()) {
            return Self::from(&s);
        }
        Expr::Call {
            function: Symbol::create(value),
            args: vec![],
        }
    }
}

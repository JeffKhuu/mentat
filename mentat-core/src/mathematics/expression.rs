use std::fmt::Display;

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

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Integer(n) => write!(f, "{n}"),
            Expr::Real(n) => write!(f, "{n}"),
            Expr::Add(exprs) => {
                let s = exprs
                    .iter()
                    .map(|expr| expr.to_string())
                    .collect::<Vec<_>>()
                    .join(" + ");
                write!(f, "({s})")
            }
            Expr::Mul(exprs) => {
                let s = exprs
                    .iter()
                    .map(|expr| expr.to_string())
                    .collect::<Vec<_>>()
                    .join(" * ");
                write!(f, "({s})")
            }
            Expr::Pow(expr, expr1) => {
                write!(f, "{expr}^{expr1}")
            }
            Expr::Neg(expr) => {
                write!(f, "-{expr}")
            }
            Expr::Call { function, args } => {
                if args.len() == 0 {
                    return write!(f, "{function}");
                }
                let s = args
                    .iter()
                    .map(|expr| expr.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{function}({s})")
            }
        }
    }
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

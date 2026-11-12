use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use thiserror::Error;

use crate::{
    evaluation::evaluator::EvaluationError,
    mathematics::symbol::Identifier,
    parser::{normalizer::Normalizer, numerical_string::NumericalString},
};

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
    Call {
        function: Identifier,
        args: Vec<Expr>,
    },
}

impl Expr {
    pub fn normalize(self, normalizer: &Normalizer) -> Self {
        match self {
            Expr::Integer(_) => normalizer.apply_rules(self),
            Expr::Real(_) => normalizer.apply_rules(self),
            Expr::Add(exprs) => {
                normalizer.apply_rules(Expr::Add(normalizer.apply_rules_vec(exprs)))
            }
            Expr::Mul(exprs) => {
                normalizer.apply_rules(Expr::Mul(normalizer.apply_rules_vec(exprs)))
            }
            Expr::Pow(expr, expr1) => normalizer.apply_rules(Expr::Pow(
                Box::new(normalizer.apply_rules(*expr)),
                Box::new(normalizer.apply_rules(*expr1)),
            )),
            Expr::Neg(expr) => {
                normalizer.apply_rules(Expr::Neg(Box::new(normalizer.apply_rules(*expr))))
            }
            Expr::Call { function, args } => normalizer.apply_rules(Expr::Call {
                function,
                args: normalizer.apply_rules_vec(args),
            }),
        }
    }

    pub fn symbols(&self) -> Vec<Identifier> {
        let mut symbols = vec![];

        match self {
            Expr::Add(exprs) | Expr::Mul(exprs) => {
                exprs
                    .iter()
                    .for_each(|expr| symbols.append(&mut expr.symbols()));
            }
            Expr::Pow(expr, expr1) => {
                symbols.append(&mut expr.symbols());
                symbols.append(&mut expr1.symbols());
            }
            Expr::Neg(expr) => {
                symbols.append(&mut expr.symbols());
            }
            Expr::Call { function, args } => {
                symbols.push(function.clone());
                args.iter()
                    .for_each(|expr| symbols.append(&mut expr.symbols()));
            }
            _ => {}
        };

        symbols
    }
}

impl Expr {
    pub fn pow(self, rhs: Self) -> Expr {
        match (self, rhs) {
            (Expr::Integer(n), Expr::Integer(m)) => Expr::Integer(n.pow(m as u32)),
            (Expr::Real(n), Expr::Integer(m)) => Expr::Real(n.powf(m as f64)),
            (Expr::Integer(n), Expr::Real(m)) => Expr::Real((n as f64).powf(m)),
            (Expr::Real(n), Expr::Real(m)) => Expr::Real(n.powf(m)),
            (lhs, rhs) => Expr::Pow(Box::new(lhs), Box::new(rhs)),
        }
    }
}

impl Add for Expr {
    type Output = Expr;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Expr::Integer(n), Expr::Integer(m)) => Expr::Integer(n + m),
            (Expr::Real(n), Expr::Integer(m)) => Expr::Real(n + m as f64),
            (Expr::Integer(n), Expr::Real(m)) => Expr::Real(n as f64 + m),
            (Expr::Real(n), Expr::Real(m)) => Expr::Real(n + m),
            (Expr::Add(mut terms), expr) => {
                terms.push(expr);
                Expr::Add(terms)
            }
            (expr, Expr::Add(mut terms)) => {
                terms.insert(0, expr);
                Expr::Add(terms)
            }
            (lhs, rhs) => Expr::Add(vec![lhs, rhs]),
        }
    }
}

impl Sub for Expr {
    type Output = Expr;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Expr::Integer(n), Expr::Integer(m)) => {
                if n > m {
                    Expr::Integer(n - m)
                } else {
                    Expr::Neg(Box::new(Expr::Integer(m - n)))
                }
            }
            (Expr::Real(n), Expr::Integer(m)) => {
                if n > m as f64 {
                    Expr::Real(n - m as f64)
                } else {
                    Expr::Neg(Box::new(Expr::Real(m as f64 - n)))
                }
            }
            (Expr::Integer(n), Expr::Real(m)) => {
                if n as f64 > m {
                    Expr::Real(n as f64 - m)
                } else {
                    Expr::Neg(Box::new(Expr::Real(m - n as f64)))
                }
            }
            (Expr::Real(n), Expr::Real(m)) => {
                if n > m {
                    Expr::Real(n - m)
                } else {
                    Expr::Neg(Box::new(Expr::Real(m - n)))
                }
            }
            (Expr::Add(mut terms), expr) => {
                terms.push(Expr::Neg(Box::new(expr)));
                Expr::Add(terms)
            }
            (expr, Expr::Add(mut terms)) => {
                terms.insert(0, Expr::Neg(Box::new(expr)));
                Expr::Add(terms)
            }
            (lhs, rhs) => Expr::Add(vec![lhs, Expr::Neg(Box::new(rhs))]),
        }
    }
}

impl Mul for Expr {
    type Output = Expr;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Expr::Integer(n), Expr::Integer(m)) => Expr::Integer(n * m),
            (Expr::Real(n), Expr::Integer(m)) => Expr::Real(n * m as f64),
            (Expr::Integer(n), Expr::Real(m)) => Expr::Real(n as f64 * m),
            (Expr::Real(n), Expr::Real(m)) => Expr::Real(n * m),
            (Expr::Mul(mut terms), expr) => {
                terms.push(expr);
                Expr::Mul(terms)
            }
            (expr, Expr::Mul(mut terms)) => {
                terms.push(expr);
                Expr::Mul(terms)
            }
            (lhs, rhs) => Expr::Mul(vec![lhs, rhs]),
        }
    }
}

impl Div for Expr {
    type Output = Result<Expr, EvaluationError>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Expr::Integer(numerator), Expr::Integer(denominator)) => {
                if denominator == 0 {
                    return Err(EvaluationError::DivisionByZero);
                };
                if numerator % denominator == 0 {
                    return Ok(Expr::Integer(numerator / denominator));
                }
                Ok(Expr::Real(numerator as f64 / denominator as f64))
            }
            (Expr::Real(numerator), Expr::Real(denominator)) => {
                if denominator == 0.0 {
                    return Err(EvaluationError::DivisionByZero);
                };
                Ok(Expr::Real(numerator / denominator))
            }
            (Expr::Integer(numerator), Expr::Real(denominator)) => {
                if denominator == 0.0 {
                    return Err(EvaluationError::DivisionByZero);
                };
                Ok(Expr::Real(numerator as f64 / denominator))
            }
            (Expr::Real(numerator), Expr::Integer(denominator)) => {
                if denominator == 0 {
                    return Err(EvaluationError::DivisionByZero);
                };
                Ok(Expr::Real(numerator / denominator as f64))
            }
            (lhs, rhs) => {
                let denominator = Expr::Pow(
                    Box::new(rhs),
                    Box::new(Expr::Neg(Box::new(Expr::Integer(1)))),
                );
                Ok(Expr::Mul(vec![lhs, denominator]))
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum ExprError {
    #[error("Failed to narrow \"{0}\" into identifier")]
    IdentifierNarrowingError(Expr),
}

impl TryInto<Identifier> for Expr {
    type Error = ExprError;

    fn try_into(self) -> Result<Identifier, Self::Error> {
        match self {
            Expr::Integer(_)
            | Expr::Real(_)
            | Expr::Add(_)
            | Expr::Mul(_)
            | Expr::Pow(_, _)
            | Expr::Neg(_) => Err(ExprError::IdentifierNarrowingError(self)),
            Expr::Call { function, .. } => Ok(function),
        }
    }
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
            function: Identifier::new(value),
            args: vec![],
        }
    }
}

mod tests {
    use rstest::*;

    #[rstest]
    #[case("2", "2", "4")]
    #[case("x", "2", "x+2")]
    #[case("2", "x", "2+x")]
    #[case("1.0", "2", "3.0")]
    #[case("3.14", "3.14", "6.28")]
    fn test_add(#[case] lhs: &str, #[case] rhs: &str, #[case] expected: &str) {
        use crate::{lang::statement::Statement::Expression, parser::parse};

        let lhs = parse(&lhs.to_string()).unwrap();
        let rhs = parse(&rhs.to_string()).unwrap();

        let expected = parse(&expected.to_string()).unwrap();

        match (lhs, rhs) {
            (Expression(lhs), Expression(rhs)) => assert_eq!(expected, Expression(lhs + rhs)),
            _ => assert!(false),
        }
    }

    #[rstest]
    #[case("2", "2", "4")]
    #[case("x", "2", "x*2")]
    #[case("2", "x", "2*x")]
    #[case("1.0", "2", "2.0")]
    #[case("3.14", "3.14", "9.8596")]
    fn test_mul(#[case] lhs: &str, #[case] rhs: &str, #[case] expected: &str) {
        use crate::{lang::statement::Statement::Expression, parser::parse};

        let lhs = parse(&lhs.to_string()).unwrap();
        let rhs = parse(&rhs.to_string()).unwrap();
        let expected = parse(&expected.to_string()).unwrap();

        match (lhs, rhs) {
            (Expression(lhs), Expression(rhs)) => assert_eq!(expected, Expression(lhs * rhs)),
            _ => assert!(false),
        }
    }
}

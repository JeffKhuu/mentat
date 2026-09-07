use crate::mathematics::symbol::Symbol;

/**
Mentat's core expression type

Examples:

2 + 2
x^2 + 2
f(x) 
*/
pub enum Expr {
    // Atomic Expressions
    Integer(i64),
    Real(f64),
    Symbol(Symbol),

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
        function: Box<Expr>,
        args: Vec<Expr>,
    },
}

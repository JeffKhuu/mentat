use crate::mathematics::{expression::Expr, symbol::Identifier};

/*
A statement is the foundational building block for Mentat
*/

#[derive(Debug, PartialEq)]
pub enum Statement {
    /// An expression that can be evaluated
    Expression(Expr),

    /// Bind an expression to an identifier for the statement's evaluation environment scope
    Assignment(Identifier, Expr),
}


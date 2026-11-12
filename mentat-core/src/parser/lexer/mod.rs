/*
The lexer is responsible for converting a raw string of text into a parse-ready intermediate representation. This intermediate representation is represented by the following rules:

The representation will be a stream of Token
A Token can be Numeric, Symbolic or Identifier

A Numeric Token is a token which represents a single number (either integer or real)

A Symbolic Token is a token which can represent a single or multicharacter symbol

An Identifier Token is a token which can represent a value for an external environment
    - A valid Identifier is a string that cannot contain symbolic characters
*/

/*
Further, a Symbolic Token can be a Operator, Delimiter or Annotation
- An Operator Symbol should classify symbols that will perform operations. For example '+', '-', '*', '/', '^', '!', '&', '|', '<', '>', '<=', '>=', '=', '!='
- An Delimiter Symbol should classify symbols that will be used to delimit groupings. For example '(', ')', '<', '>', '{', '}'
- An Annotation Symbol should classify symbols that will be used to describe or modify statements. For example '->', '=>', ':', ':=', ','

Since symbolic tokens can be ambiguous ('<' as operator vs. '<' as open angle bracket) we need to parse into a second contextualize intermediate representation before parsing into our grammar's specific synax. The full pipeline for lexing/parsing now looks like:

Raw String -> Tokens -> ContextualizedTokens -> Statements
*/

use std::fmt::Display;

use crate::{
    parser::{lexer::symbol::Symbol, numerical_string::NumericalString},
};

pub mod lexer;
pub mod symbol;

#[derive(Debug, PartialEq)]
pub(crate) enum TokenKind {
    Numeric(NumericalString),
    Symbolic(Symbol),
    Identifying(String),
}

#[derive(Debug)]
pub(crate) enum TokenPattern {
    Numeric,
    Identifying,
    Symbol(Symbol),
}

impl Display for TokenPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenPattern::Numeric => write!(f, "number"),
            TokenPattern::Identifying => write!(f, "identifier"),
            TokenPattern::Symbol(symbol) => write!(f, "{symbol}"),
        }
    }
}

impl Into<String> for TokenPattern {
    fn into(self) -> String {
        match self {
            TokenPattern::Numeric => String::from("number"),
            TokenPattern::Identifying => String::from("identifier"),
            TokenPattern::Symbol(symbol) => (&symbol).into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Span {
    pub start: usize, // Inclusive
    pub end: usize,   // Exclusive
}

#[derive(Debug, PartialEq)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Into<String> for &TokenKind {
    fn into(self) -> String {
        match self {
            TokenKind::Numeric(numerical_string) => numerical_string.to_string(),
            TokenKind::Symbolic(symbol) => (symbol).into(),
            TokenKind::Identifying(iden) => iden.to_string(),
        }
    }
}

impl Into<String> for &Token {
    fn into(self) -> String {
        (&self.kind).into()
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Numeric(numerical_string) => write!(f, "{numerical_string}"),
            TokenKind::Symbolic(symbol) => write!(f, "{symbol}"),
            TokenKind::Identifying(identifier) => write!(f, "{identifier}"),
        }
    }
}

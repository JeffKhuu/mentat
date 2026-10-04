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
- An Operator Symbol should classify symbols that will perform operations. For example '+', '-', '/', '^', '!', '&', '|', '<', '>', '<=', '>=', '=', '!='
- An Delimiter Symbol should classify symbols that will be used to delimit groupings. For example '(', ')', '<', '>', '{', '}'
- An Annotation Symbol should classify symbols that will be used to describe or modify statements. For example '->', '=>', ':', ':=', ','

Since symbolic tokens can be ambiguous ('<' as operator vs. '<' as open angle bracket) we need to parse into a second contextualize intermediate representation before parsing into our grammar's specific synax. The full pipeline for lexing/parsing now looks like:

Raw String -> Tokens -> ContextualizedTokens -> Statements
*/

use crate::parser::{new_lexer::symbol::Symbol, numerical_string::NumericalString};

pub mod lexer;
pub(crate) mod symbol;

#[derive(Debug, PartialEq)]
pub(crate) enum TokenKind {
    Numeric(NumericalString),
    Symbolic(Symbol),
    Identifying(String),
}

#[derive(Debug)]
pub(crate) struct Span {
    start: usize, // Inclusive
    end: usize,   // Exclusive
}

#[derive(Debug)]
pub(crate) struct Token {
    kind: TokenKind,
    span: Span,
}

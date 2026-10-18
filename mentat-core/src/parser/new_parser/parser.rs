use thiserror::Error;

use crate::parser::{lexer::Token, new_lexer::TokenKind};

pub struct Parser {
    token: Vec<Token>,
}

enum ParserState {
    Matches(TokenKind),
    ExpectOperand,
    ExpectOperator,
}

#[derive(Debug, Error)]
pub enum ParseError {}

impl Parser {}

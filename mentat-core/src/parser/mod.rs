/*
The goal of the parser should be to take a string of user input and eventually produce a Mentat-valid expression
*/

use itertools::peek_nth;

use crate::{lang::statement::Statement, mathematics::expression::Expr, parser::{lexer::lexer::Lexer, parser::parser::{ParseError, Parser}}};

mod lexer;
pub mod normalizer;
pub(crate) mod numerical_string;
pub mod parser;

pub fn parse(line: &String) -> Result<Statement, ParseError> {
    let lexer = Lexer::new(line);
    let mut parser = Parser::new(peek_nth(lexer.tokenize().into_iter().flatten()));
    parser.parse()
}
pub fn parse_expr(line: &String) -> Result<Expr, ParseError> {
    let lexer = Lexer::new(line);
    let mut parser = Parser::new(peek_nth(lexer.tokenize().into_iter().flatten()));
    parser.expression()
}

/*
The goal of the parser should be to take a string of user input and eventually produce a Mentat-valid expression
*/

mod lexer;
pub mod normalizer;
pub(crate) mod numerical_string;
pub mod parser;

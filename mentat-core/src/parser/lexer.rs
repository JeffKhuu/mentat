/*
The lexer should take as input a raw string representing the expression, sanitize the input of whitespace and create a tokenized form.

Rules for tokens:
A token cannot contain whitespace (spaces, tabs, etc.)
A numerical token is one or more digits followed optionally by a '.' and one or more digits
An identifier is an alphabetic string of one or more characters
+, -, *, /, ^ are all valid symbols
all other symbols are invalid
*/

use std::{iter::Peekable, str::Chars};

use regex::Regex;
use thiserror::Error;

use crate::parser::{lexer::SymbolToken::Operator, numerical_string::NumericalString};

/**
Notes for future development:
====
- Currently our Token enum is very primitive, it might be best to practice to create a TokenKind enum and
    for Token to be a struct that contains information relating to the raw string (start and end) (for debugging and error handling purposes)
- A more comprehensive error tree could be developed

*/

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Token {
    // Atomic Literals
    Number(NumericalString),
    Identifier(String),
    Symbol(SymbolToken),
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::Number(numerical_string) => numerical_string.to_string(),
            Token::Identifier(s) => s.to_string(),
            Token::Symbol(symbol_token) => symbol_token.to_string(),
        }
    }
}

impl From<OperatorToken> for Token {
    fn from(value: OperatorToken) -> Self {
        Token::Symbol(SymbolToken::Operator(value))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum SymbolToken {
    Operator(OperatorToken),
    Comma,
    Paren(Parenthesis),
}

impl ToString for SymbolToken {
    fn to_string(&self) -> String {
        match self {
            Operator(operator_token) => operator_token.to_string(),
            SymbolToken::Comma => String::from(","),
            SymbolToken::Paren(parenthesis) => parenthesis.to_string(),
        }
    }
}

impl TryFrom<&char> for SymbolToken {
    type Error = TokenizationError;

    fn try_from(c: &char) -> Result<Self, Self::Error> {
        use SymbolToken::*;
        if let Ok(op) = OperatorToken::try_from(c) {
            return Ok(Operator(op));
        }
        if *c == ',' {
            return Ok(Comma);
        }

        if let Ok(paren) = Parenthesis::try_from(c) {
            return Ok(Paren(paren));
        }
        Err(TokenizationError::UnexpectedSymbol(*c))
    }
}

impl Into<Token> for SymbolToken {
    fn into(self) -> Token {
        match self {
            Operator(operator_token) => Token::Symbol(Operator(operator_token)),
            SymbolToken::Comma => Token::Symbol(SymbolToken::Comma),
            SymbolToken::Paren(parenthesis) => Token::Symbol(SymbolToken::Paren(parenthesis)),
        }
    }
}

// TODO: We might consider implementing the into<Token> trait
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum OperatorToken {
    Plus,  // +
    Minus, // -
    Star,  // *
    Slash, // /
    Caret, // ^

    UnaryPlus,  // +
    UnaryMinus, // -
}

impl OperatorToken {
    fn precedence(&self) -> u8 {
        match self {
            Self::Plus | Self::Minus => 1,
            Self::Star | Self::Slash => 2,
            Self::UnaryPlus | Self::UnaryMinus => 3,
            Self::Caret => 4,
        }
    }

    fn left_associative(&self) -> bool {
        match self {
            Self::Caret | Self::UnaryPlus | Self::UnaryMinus => false,

            Self::Plus | Self::Minus | Self::Star | Self::Slash => true,
        }
    }

    pub(crate) fn should_pop_before(&self, other: &OperatorToken) -> bool {
        self.precedence() > other.precedence()
            || (self.precedence() == other.precedence() && other.left_associative())
    }
}

impl ToString for OperatorToken {
    fn to_string(&self) -> String {
        match self {
            OperatorToken::Plus | OperatorToken::UnaryPlus => String::from("+"),
            OperatorToken::Minus | OperatorToken::UnaryMinus => String::from("-"),
            OperatorToken::Star => String::from("*"),
            OperatorToken::Slash => String::from("/"),
            OperatorToken::Caret => String::from("^"),
        }
    }
}

impl TryFrom<&char> for OperatorToken {
    type Error = TokenizationError;

    fn try_from(c: &char) -> Result<Self, Self::Error> {
        use OperatorToken::*;
        match c {
            '+' => Ok(Plus),
            '-' => Ok(Minus),
            '*' => Ok(Star),
            '/' => Ok(Slash),
            '^' => Ok(Caret),
            _ => Err(TokenizationError::UnexpectedSymbol(*c)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Parenthesis {
    Left,  // (
    Right, // )
}

impl ToString for Parenthesis {
    fn to_string(&self) -> String {
        match self {
            Parenthesis::Left => String::from("("),
            Parenthesis::Right => String::from(")"),
        }
    }
}

impl TryFrom<&char> for Parenthesis {
    type Error = TokenizationError;

    fn try_from(c: &char) -> Result<Self, Self::Error> {
        match c {
            '(' => Ok(Parenthesis::Left),
            ')' => Ok(Parenthesis::Right),
            _ => Err(TokenizationError::UnexpectedSymbol(*c)),
        }
    }
}

#[derive(Debug, Error)]
pub enum TokenizationError {
    #[error("Unexpected symbol '{0}'.")]
    UnexpectedSymbol(char),
}

/// Tokenize takes a raw string and returns an iterable collection of Tokens
/// if the string can be tokenized into a collection of valid tokens otherwise, a corresponding TokenizationError is returned
pub(crate) fn tokenize(raw: &String) -> Result<impl Iterator<Item = Token>, TokenizationError> {
    let mut tokens = vec![];
    for s in sanitize(&raw).iter() {
        if let Some(num) = NumericalString::create(s) {
            tokens.push(Token::Number(num));
            continue;
        }

        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.peek() {
            // This is a primitive implementation since all our symbols are one char
            if let Ok(symbol) = SymbolToken::try_from(ch) {
                tokens.push(Token::Symbol(symbol));
                chars.next();
                continue;
            }
            match ch {
                c if c.is_ascii_alphabetic() => {
                    tokens.push(scan_identifier(&mut chars)?);
                }
                _ => return Err(TokenizationError::UnexpectedSymbol(*ch)),
            }
        }
    }
    Ok(tokens.into_iter())
}

/// scan_identifier takes a peekable stream of characters and should consume ONE token worth
/// of characters that is an identifier.
/// Side Effects: chars is mutated to remove one token
fn scan_identifier(chars: &mut Peekable<Chars<'_>>) -> Result<Token, TokenizationError> {
    let mut str: String = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_alphabetic() {
            str.push(*ch);
            chars.next();
            continue;
        }
        break;
    }
    Ok(Token::Identifier(str))
}

/// sanitize takes a raw string and splits it by whitespace and a numeric/non-numeric separation.
/// Each element of the resulting vector is guaranteed to be a string of symbols or a number
fn sanitize(raw: &str) -> Vec<&str> {
    raw.split_whitespace()
        .map(|s| split_numbers(s))
        .flatten()
        .collect()
}

/// split_numbers splits the given string into numeric and non-numeric components
/// Example:
/// split_numbers("123abc456") -> \["123", "abc", "456"\]
/// split_numbers("3.1415pi") -> \["3.1415", "pi"\]
fn split_numbers(s: &str) -> Vec<&str> {
    let regex = Regex::new(r"\d+(?:\.\d+)?|[^\d]+").unwrap();
    regex.find_iter(s).map(|n| n.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use crate::parser::lexer::{OperatorToken::*, SymbolToken::*};

    use super::{Token::*, *};
    use rstest::*;

    #[rstest]
    #[case::variable("x+123", Identifier("x".to_string()))]
    #[case::function("f(x) = x^2", Identifier("f".to_string()))]
    #[case::trascendental("sin(pi)", Identifier("sin".to_string()))]
    #[case::constant("pi", Identifier("pi".to_string()))]
    #[case::name("num", Identifier("num".to_string()))]
    fn test_scan_identifier(#[case] raw: &str, #[case] expected: Token) {
        if let Ok(token) = scan_identifier(&mut raw.chars().peekable()) {
            assert_eq!(expected, token);
        }
    }

    #[rstest]
    #[case("123+456", vec!["123", "+", "456"])]
    #[case("x - 50", vec!["x", "-", "50"])]
    #[case("12 /4", vec!["12", "/", "4"])]
    #[case("x^2+sin(x)", vec!["x^", "2", "+sin(x)"])]
    #[case("2+2", vec!["2", "+", "2"])]
    fn test_sanitize(#[case] raw: &str, #[case] expected: Vec<&str>) {
        assert_eq!(expected, sanitize(raw))
    }

    fn int(n: i64) -> Token {
        Token::Number(NumericalString::from(n))
    }
    fn real(n: f64) -> Token {
        Token::Number(NumericalString::from(n))
    }
    fn iden(s: &str) -> Token {
        Token::Identifier(s.to_string())
    }
    fn op(op: OperatorToken) -> Token {
        Token::Symbol(Operator(op))
    }
    fn lparen() -> Token {
        Token::Symbol(Paren(Parenthesis::Left))
    }
    fn rparen() -> Token {
        Token::Symbol(Paren(Parenthesis::Right))
    }

    #[rstest]
    #[case("2+2", vec![ int(2), op(Plus), int(2)])]
    #[case("x+y", vec![ iden("x"), op(Plus), iden("y")])]
    #[case("3.14*pi", vec![ real(3.14), op(Star), iden("pi")])]
    #[case("y^2", vec![ iden("y"), op(Caret), int(2)])]
    #[case("+", vec![op(Plus)])]
    #[case("sin", vec![iden("sin")])]
    #[case("x", vec![iden("x")])]
    #[case("sin(x)/tan(x)", vec![iden("sin"), lparen(), iden("x"), rparen(), op(Slash), iden("tan"), lparen(), iden("x"), rparen()])]
    #[case("f(x)", vec![iden("f"), lparen(), iden("x"), rparen()])]
    #[case("500^2", vec![int(500), op(Caret), int(2)])]
    #[case("100.12345678", vec![real(100.12345678)])]
    fn test_tokenize(#[case] raw: &str, #[case] expected: Vec<Token>) {
        assert_eq!(
            expected,
            tokenize(&raw.to_string()).unwrap().collect::<Vec<_>>()
        )
    }

    // NOTE: This unit test will most likely change over time as more symbols are supported
    #[rstest]
    #[case::hash("1#2")]
    #[case::bracket("[sin(x)]")]
    fn test_invalid_tokenize(#[case] raw: &str) {
        assert!(tokenize(&raw.to_string()).is_err())
    }
}

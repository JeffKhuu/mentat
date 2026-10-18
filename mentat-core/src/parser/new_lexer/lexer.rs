use thiserror::Error;

use crate::parser::{
    new_lexer::{
        Span, Token, TokenKind,
        symbol::{Parentheses, Symbol},
    },
    numerical_string::NumericalString,
};

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    input: &'a str,
}

#[derive(Debug, Error)]
pub enum LexError {
    #[error("Unexpected character found {0}.")]
    Unexpected(char),
    #[error("Invalid number: {0}")]
    InvalidNumber(#[from] NumberError),
}

#[derive(Debug, Error)]
pub enum NumberError {
    #[error("Found multiple decimal points in number.")]
    MultipleDecimals(Span),
    #[error("Failed to create number.")]
    InvalidNumber(Span),
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars = input.char_indices().peekable();
        Self { chars, input }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while let Some(&char_indice) = self.chars.peek() {
            // char_indice is (usize, char)
            match char_indice.1 {
                c if c.is_whitespace() => {
                    self.chars.next();
                }
                c if c.is_ascii_digit() || c == '.' => tokens.push(self.number()?),
                c if c.is_alphabetic() => tokens.push(self.identifier()?),
                '+' => tokens.push(self.consume_symbol(TokenKind::Symbolic(Symbol::Plus))?),
                '-' => tokens.push(self.dashed_symbol()?),
                '*' => tokens.push(self.consume_symbol(TokenKind::Symbolic(Symbol::Star))?),
                '/' => tokens.push(self.consume_symbol(TokenKind::Symbolic(Symbol::Slash))?),
                '^' => tokens.push(self.consume_symbol(TokenKind::Symbolic(Symbol::Caret))?),
                '(' => {
                    let sym = TokenKind::Symbolic(Symbol::Parentheses(Parentheses::Open));
                    tokens.push(self.consume_symbol(sym)?)
                }
                ')' => {
                    let sym = TokenKind::Symbolic(Symbol::Parentheses(Parentheses::Closed));
                    tokens.push(self.consume_symbol(sym)?)
                }
                ',' => tokens.push(self.consume_symbol(TokenKind::Symbolic(Symbol::Comma))?),
                c => return Err(LexError::Unexpected(c)),
            }
        }

        Ok(tokens)
    }

    fn consume(&mut self, expected: char) -> bool {
        if let Some((_, ch)) = self.chars.peek() {
            if ch == &expected {
                self.chars.next();
                return true;
            }
            return false;
        }
        false
    }

    fn number(&mut self) -> Result<Token, LexError> {
        let start = self.position();
        let mut decimal_seen = false;

        while let Some((_, ch)) = self.chars.peek() {
            match ch {
                '0'..='9' => {
                    self.chars.next();
                }
                '.' if !decimal_seen => {
                    decimal_seen = true;
                    self.chars.next();
                }
                '.' => {
                    let end = self.position();
                    return Err(NumberError::MultipleDecimals(Span { start, end }).into());
                }
                _ => break,
            }
        }

        let end = self.position();
        Ok(Token {
            kind: TokenKind::Numeric(
                NumericalString::create(&self.input[start..end])
                    .ok_or_else(|| NumberError::InvalidNumber(Span { start, end }))?,
            ),
            span: Span { start, end },
        })
    }

    fn identifier(&mut self) -> Result<Token, LexError> {
        let start = self.position();
        while let Some((_, ch)) = self.chars.peek() {
            match ch {
                ch if ch.is_alphabetic() => {
                    self.chars.next();
                }
                _ => break,
            }
        }

        let end = self.position();
        Ok(Token {
            kind: TokenKind::Identifying(self.input[start..end].to_string()),
            span: Span { start, end },
        })
    }

    fn dashed_symbol(&mut self) -> Result<Token, LexError> {
        let start = self.position();
        self.chars.next();

        if self.consume('>') {
            let end = self.position();
            Ok(Token {
                kind: TokenKind::Symbolic(Symbol::ThinArrow),
                span: Span { start, end },
            })
        } else {
            let end = self.position();
            Ok(Token {
                kind: TokenKind::Symbolic(Symbol::Minus),
                span: Span { start, end },
            })
        }
    }

    fn consume_symbol(&mut self, token: TokenKind) -> Result<Token, LexError> {
        let span = self.span(1);
        self.chars.next();
        Ok(Token { kind: token, span })
    }

    /// get the index of the next character
    fn position(&mut self) -> usize {
        match self.chars.peek() {
            Some((ind, _)) => *ind,
            None => self.input.len(),
        }
    }

    /// get a Span of size n from the next character
    fn span(&mut self, n: usize) -> Span {
        let start = self.position();
        let end = start + n;
        Span { start, end }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    #[cfg(test)]
    use crate::parser::new_lexer::Token;
    use crate::parser::{
        new_lexer::{
            Span, TokenKind,
            lexer::Lexer,
            symbol::{Parentheses, Symbol},
        },
        numerical_string::NumericalString,
    };

    #[rstest]
    #[case("123+456", TokenKind::Numeric(NumericalString::from(123)))]
    #[case("100.1", TokenKind::Numeric(NumericalString::from(100.1)))]
    #[case("100.111", TokenKind::Numeric(NumericalString::from(100.111)))]
    #[case("67.69,15", TokenKind::Numeric(NumericalString::from(67.69)))]
    #[case("15,67.69", TokenKind::Numeric(NumericalString::from(15)))]
    fn test_consume_number(#[case] raw: &str, #[case] expected: TokenKind) {
        let mut lexer = Lexer::new(raw);
        let result = lexer.number().expect("Failed to consume number token");

        dbg!(result.span);
        assert_eq!(result.kind, expected);
    }
    #[rstest]
    #[case("x", TokenKind::Identifying("x".to_string()))]
    #[case("sin", TokenKind::Identifying("sin".to_string()))]
    #[case("sin123", TokenKind::Identifying("sin".to_string()))]
    #[case("e", TokenKind::Identifying("e".to_string()))]
    #[case("a+b", TokenKind::Identifying("a".to_string()))]
    fn test_consume_identifier(#[case] raw: &str, #[case] expected: TokenKind) {
        let mut lexer = Lexer::new(raw);
        let result = lexer.identifier().expect("Failed to consume number token");

        dbg!(result.span);
        assert_eq!(result.kind, expected);
    }

    fn symb(symbol: Symbol, start: usize, end: usize) -> Token {
        Token {
            kind: TokenKind::Symbolic(symbol),
            span: Span { start, end },
        }
    }
    fn iden(iden: &str, start: usize, end: usize) -> Token {
        Token {
            kind: TokenKind::Identifying(iden.to_string()),
            span: Span { start, end },
        }
    }
    fn num(num: u32, start: usize, end: usize) -> Token {
        Token {
            kind: TokenKind::Numeric(NumericalString::from(num)),
            span: Span { start, end },
        }
    }

    #[rstest]
    #[case("x+y", vec![iden("x", 0, 1), symb(Symbol::Plus, 1, 2), iden("y", 2, 3)])]
    #[case("(x+y)", vec![symb(Symbol::Parentheses(Parentheses::Open), 0, 1), iden("x", 1, 2), symb(Symbol::Plus, 2, 3), iden("y", 3, 4),symb(Symbol::Parentheses(Parentheses::Closed), 4, 5)])]
    #[case("sin(pi)", vec![iden("sin", 0, 3), symb(Symbol::Parentheses(Parentheses::Open), 3, 4), iden("pi", 4, 6), symb(Symbol::Parentheses(Parentheses::Closed), 6, 7)])]
    #[case("-5", vec![symb(Symbol::Minus, 0, 1), num(5, 1, 2)])]
    fn test_lex(#[case] raw: &str, #[case] expected: Vec<Token>) {
        let lexer = Lexer::new(raw);
        let result = lexer.tokenize().expect("Failed to parse test case");

        assert_eq!(result, expected)
    }
}

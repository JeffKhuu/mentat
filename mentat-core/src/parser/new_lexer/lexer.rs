use thiserror::Error;

use crate::parser::{
    new_lexer::{Span, Token, TokenKind, symbol::Symbol},
    numerical_string::NumericalString,
};

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    input: &'a str,
    pos: usize,
}

#[derive(Debug, Error)]
pub enum LexError {
    #[error("Unexpected Token Found.")]
    Unexpected(Token),
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
        Self {
            chars,
            input,
            pos: 0,
        }
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
                '+' => {
                    let span = self.span(1);
                    self.chars.next();
                    tokens.push(Token {
                        kind: TokenKind::Symbolic(Symbol::Plus),
                        span,
                    });
                }
                '-' => tokens.push(self.dashed_symbol()?),
                _ => {}
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

    use crate::parser::{
        new_lexer::{TokenKind, lexer::Lexer},
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
}

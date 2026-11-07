use std::iter::Peekable;

use itertools::PeekNth;
use thiserror::Error;

use crate::{
    lang::statement::Statement,
    mathematics::expression::Expr,
    parser::new_lexer::{Token, TokenKind, symbol::Symbol},
};

pub struct Parser<I: Iterator<Item = Token>> {
    tokens: PeekNth<I>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Found an unexpected token \"{token}\" at position {pos}")]
    UnexpectedToken { token: String, pos: usize },
    #[error("Found an expected end to expression")]
    UnexpectedEnd,
}

enum TokenPattern {
    Numeric,
    Identifying,
    Symbol(Symbol),
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(tokens: PeekNth<I>) -> Self {
        Parser { tokens }
    }

    fn matches(&mut self, patterns: &[TokenPattern]) -> Option<Token> {
        let kind = &self.tokens.peek()?.kind;

        let matched = patterns.iter().any(|pattern| match (pattern, kind) {
            (TokenPattern::Numeric, TokenKind::Numeric(_)) => true,
            (TokenPattern::Identifying, TokenKind::Identifying(_)) => true,
            (TokenPattern::Symbol(expected), TokenKind::Symbolic(actual)) => expected == actual,
            _ => false,
        });

        if matched {
            Some(self.tokens.next()?)
        } else {
            None
        }
    }

    fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        if let Some(token) = self.tokens.peek_nth(1) {
            if token.kind == TokenKind::Symbolic(Symbol::Assignment) {
                return self.assignment();
            }
        }
        self.term()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        todo!()
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut summands: Vec<Expr> = vec![self.factor()?];
        while let Some(operator) = self.matches(&[
            TokenPattern::Symbol(Symbol::Plus),
            TokenPattern::Symbol(Symbol::Minus),
        ]) {
            let mut rhs = self.factor()?;
            if operator.kind == TokenKind::Symbolic(Symbol::Minus) {
                rhs = Expr::Neg(Box::new(rhs));
            }
            summands.push(rhs);
        }

        if summands.len() == 1 {
            return Ok(summands.pop().unwrap());
        }
        Ok(Expr::Add(summands))
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut factors: Vec<Expr> = vec![self.unary()?];

        while let Some(operator) = self.matches(&[
            TokenPattern::Symbol(Symbol::Star),
            TokenPattern::Symbol(Symbol::Slash),
        ]) {
            let mut rhs = self.unary()?;
            if operator.kind == TokenKind::Symbolic(Symbol::Slash) {
                rhs = Expr::Pow(
                    Box::new(rhs),
                    Box::new(Expr::Neg(Box::new(Expr::Integer(1)))),
                );
            }
            factors.push(rhs);
        }

        if factors.len() == 1 {
            return Ok(factors.pop().unwrap());
        }

        Ok(Expr::Mul(factors))
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(_) = self.matches(&[TokenPattern::Symbol(Symbol::Minus)]) {
            Ok(Expr::Neg(Box::new(self.unary()?)))
        } else {
            self.literal()
        }
    }

    fn literal(&mut self) -> Result<Expr, ParseError> {
        if let Some(token) = self.matches(&[TokenPattern::Identifying, TokenPattern::Numeric]) {
            match token.kind {
                TokenKind::Numeric(numerical_string) => Ok(Expr::from(&numerical_string)),
                TokenKind::Identifying(iden) => Ok(Expr::from(iden)),
                TokenKind::Symbolic(_) => unreachable!(),
            }
            // TODO: ADD SUPPORT FOR PARENS

            // TODO: ADD SUPPORT FOR FUNCTIONS
        } else {
            match self.tokens.peek() {
                Some(token) => Err(ParseError::UnexpectedToken {
                    token: token.into(),
                    pos: token.span.start,
                }),
                None => Err(ParseError::UnexpectedEnd),
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{
        mathematics::{expression::Expr, symbol::Identifier},
        parser::new_parser::parser::Parser,
    };

    fn e_int(n: u64) -> Expr {
        Expr::Integer(n)
    }
    fn e_real(n: f64) -> Expr {
        Expr::Real(n)
    }
    fn e_iden(s: &str) -> Expr {
        e_func(s, &[])
    }
    fn neg(e: Expr) -> Expr {
        Expr::Neg(Box::new(e))
    }
    fn pow(e1: Expr, e2: Expr) -> Expr {
        Expr::Pow(Box::new(e1), Box::new(e2))
    }
    fn reciprocal(e: Expr) -> Expr {
        pow(e, neg(e_int(1)))
    }
    fn add(es: &[Expr]) -> Expr {
        Expr::Add(es.to_vec())
    }
    fn mul(es: &[Expr]) -> Expr {
        Expr::Mul(es.to_vec())
    }
    fn e_func(ident: &str, args: &[Expr]) -> Expr {
        Expr::Call {
            function: Identifier::from(ident),
            args: args.to_vec(),
        }
    }

    #[rstest]
    #[case("1", e_int(1))]
    #[case("-1", neg(e_int(1)))]
    #[case("1+1", add(&[e_int(1), e_int(1)]))]
    #[case("1+2+3", add(&[e_int(1), e_int(2), e_int(3)]))]
    #[case("1-2+3", add(&[e_int(1), neg(e_int(2)), e_int(3)]))]
    #[case("2*5", mul(&[e_int(2), e_int(5)]))]
    #[case("2/5", mul(&[e_int(2), reciprocal(e_int(5))]))]
    #[case("3.14159", e_real(3.14159))]
    fn test_parse(#[case] raw: &str, #[case] expected: Expr) {
        use itertools::peek_nth;

        use crate::parser::new_lexer::lexer::Lexer;

        let lexer = Lexer::new(raw);
        let mut parser = Parser::new(peek_nth(lexer.tokenize().into_iter().flatten()));
        assert_eq!(expected, parser.parse().unwrap())
    }
}

use std::unreachable;

use itertools::PeekNth;
use thiserror::Error;

use crate::{
    lang::statement::Statement,
    mathematics::{
        expression::{Expr, ExprError},
        symbol::Identifier,
    },
    parser::lexer::{
        Token, TokenKind, TokenPattern,
        symbol::{Parentheses, Symbol},
    },
};

pub struct Parser<I: Iterator<Item = Token>> {
    tokens: PeekNth<I>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Found an unexpected token \"{token}\" at position {pos}")]
    UnexpectedToken { token: String, pos: usize },
    #[error("Expected \"{expected}\" but found \"{token}\" at position {pos}.")]
    MismatchedToken {
        token: String,
        expected: String,
        pos: usize,
    },
    #[error("Unexpectedly reached end of expression")]
    UnexpectedEnd,
    #[error("An error occured: {0}")]
    ExprError(#[from] ExprError),
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(tokens: PeekNth<I>) -> Self {
        Parser { tokens }
    }

    /// Consumes the given pattern from the parser's iterator. Error if the next token does not
    /// match the expected pattern.
    fn consume(&mut self, expected: TokenPattern) -> Result<Token, ParseError> {
        let token = self.tokens.peek().ok_or(ParseError::UnexpectedEnd)?;
        let kind = &token.kind;
        let matched = match (&expected, kind) {
            (TokenPattern::Numeric, TokenKind::Numeric(_)) => true,
            (TokenPattern::Identifying, TokenKind::Identifying(_)) => true,
            (TokenPattern::Symbol(expected_symbol), TokenKind::Symbolic(actual_symbol)) => {
                expected_symbol == actual_symbol
            }
            _ => false,
        };

        if matched {
            return Ok(self.tokens.next().unwrap());
        } else {
            return Err(ParseError::MismatchedToken {
                token: token.into(),
                expected: expected.into(),
                pos: token.span.start,
            });
        }
    }

    /// Peek the next token in the parsers iterator with a given list of patterns, token remains in
    /// the iterator if matched
    fn peek(&mut self, patterns: &[TokenPattern]) -> Option<&Token> {
        let token = &self.tokens.peek()?;
        let kind = &token.kind;

        let matched = patterns.iter().any(|pattern| match (pattern, kind) {
            (TokenPattern::Numeric, TokenKind::Numeric(_)) => true,
            (TokenPattern::Identifying, TokenKind::Identifying(_)) => true,
            (TokenPattern::Symbol(expected), TokenKind::Symbolic(actual)) => expected == actual,
            _ => false,
        });

        if matched { Some(token) } else { None }
    }

    /// Matches the next token in the parsers iterator with a given list of patterns, removes the
    /// matched token if present
    fn matches(&mut self, patterns: &[TokenPattern]) -> Option<Token> {
        if self.peek(patterns).is_some() {
            Some(self.tokens.next()?)
        } else {
            None
        }
    }

    pub fn parse(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        if self
            .matches(&[TokenPattern::Symbol(Symbol::Assignment)])
            .is_some()
        {
            let iden: Identifier = expr.try_into()?;
            let rhs = self.expression()?;
            return Ok(Statement::Assignment(iden, rhs));
        } else {
            return Ok(Statement::Expression(expr));
        }
    }

    pub(crate) fn expression(&mut self) -> Result<Expr, ParseError> {
        self.term()
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let expr = self.factor()?;
        let mut summands: Vec<Expr> = vec![];
        match expr {
            Expr::Add(exprs) => summands = exprs,
            _ => summands.push(expr),
        }
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
        let expr = self.unary()?;
        let mut factors: Vec<Expr> = vec![];
        match expr {
            Expr::Mul(exprs) => factors = exprs,
            _ => factors.push(expr),
        }

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
            self.exponential()
        }
    }

    fn exponential(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.literal()?;
        if let Some(_) = self.matches(&[TokenPattern::Symbol(Symbol::Caret)]) {
            return Ok(Expr::Pow(Box::new(lhs), Box::new(self.expression()?)));
        }
        Ok(lhs)
    }

    /// parse the args of the given identifier, args are defined to be parenthetically-enclosed
    /// comma-separated expressions
    fn args_of(&mut self, identifier: String) -> Result<Expr, ParseError> {
        let mut args: Vec<Expr> = vec![];

        self.consume(TokenPattern::Symbol(Symbol::Parentheses(Parentheses::Open)))?;
        while self
            .matches(&[TokenPattern::Symbol(Symbol::Parentheses(
                Parentheses::Closed,
            ))])
            .is_none()
        {
            args.push(self.expression()?);
            if self
                .matches(&[TokenPattern::Symbol(Symbol::Parentheses(
                    Parentheses::Closed,
                ))])
                .is_some()
            {
                break;
            } else {
                self.consume(TokenPattern::Symbol(Symbol::Comma))?;
            }
        }
        Ok(Expr::Call {
            function: Identifier::new(identifier),
            args,
        })
    }

    fn function(&mut self, iden: String) -> Result<Expr, ParseError> {
        match self.peek(&[TokenPattern::Symbol(Symbol::Parentheses(Parentheses::Open))]) {
            // The identifier is a function, we must parse its arguments now
            Some(_) => Ok(self.args_of(iden)?),
            None => Ok(Expr::from(iden)),
        }
    }

    /// parse and return a single literal value or sub-expression (identifier, number, function,
    /// parenthesized expression)
    fn literal(&mut self) -> Result<Expr, ParseError> {
        if let Some(token) = self.matches(&[
            TokenPattern::Identifying,
            TokenPattern::Numeric,
            TokenPattern::Symbol(Symbol::Parentheses(Parentheses::Open)),
        ]) {
            match token.kind {
                TokenKind::Numeric(numerical_string) => Ok(Expr::from(&numerical_string)),
                TokenKind::Identifying(iden) => self.function(iden),
                TokenKind::Symbolic(ref symbol) => match symbol {
                    Symbol::Parentheses(parentheses) => match parentheses {
                        Parentheses::Open => {
                            let paren_expr = self.expression()?;
                            self.consume(TokenPattern::Symbol(Symbol::Parentheses(
                                Parentheses::Closed,
                            )))?;
                            Ok(paren_expr)
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                },
            }
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
        lang::statement::Statement,
        mathematics::{expression::Expr, symbol::Identifier},
        parser::parser::parser::Parser,
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

    fn assign(ident: &str, expr: Expr) -> Statement {
        Statement::Assignment(Identifier::from(ident), expr)
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
    #[case("(2 * pi) * r", mul(&[e_int(2), e_iden("pi"), e_iden("r")]))]
    #[case("(2 * pi) + r", add(&[mul(&[e_int(2), e_iden("pi")]), e_iden("r")]))]
    #[case("f(x)", e_func("f", &[e_iden("x")]))]
    #[case("f(x) + 5", add(&[e_func("f", &[e_iden("x")]), e_int(5)]))]
    #[case("(-x)^2", pow(neg(e_iden("x")), e_int(2)))]
    #[case("5 * -(-x)^2", mul(&[e_int(5), neg(pow(neg(e_iden("x")), e_int(2)))]))]
    #[case("x^2", pow(e_iden("x"), e_int(2)))]
    fn test_parse_expr(#[case] raw: &str, #[case] expected: Expr) {
        use itertools::peek_nth;

        use crate::{lang::statement::Statement, parser::lexer::lexer::Lexer};

        let expected = Statement::Expression(expected);

        let lexer = Lexer::new(raw);
        let mut parser = Parser::new(peek_nth(lexer.tokenize().into_iter().flatten()));
        assert_eq!(expected, parser.parse().unwrap())
    }

    #[rstest]
    #[case("f(x) := 2 * x", assign("f", mul(&[e_int(2), e_iden("x")])))]
    #[case("x := 2 ", assign("x", e_int(2)))]
    fn test_parse_assignment(#[case] raw: &str, #[case] expected: Statement) {
        use itertools::peek_nth;

        use crate::parser::lexer::lexer::Lexer;

        let lexer = Lexer::new(raw);
        let mut parser = Parser::new(peek_nth(lexer.tokenize().into_iter().flatten()));
        assert_eq!(expected, parser.parse().unwrap())
    }
}

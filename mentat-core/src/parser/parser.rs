/*
The parser should take as input an iterable collection of tokens and return the
abstract syntax tree as a valid math expression if the tokens represent a valid expression.
*/

use std::collections::VecDeque;

use thiserror::Error;

use crate::{
    mathematics::{expression::Expr, symbol::Symbol},
    parser::lexer::*,
};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to tokenize expression: {0}")]
    TokenizationError(#[from] TokenizationError),
    #[error("Unexpected token: {0}.")]
    UnexpectedToken(String),
    #[error("Unexpected end of expression.")]
    UnexpectedEndOfExpr,
    #[error("Unexpected comma.")]
    UnexpectedComma,
    #[error("Mismatched parentheses.")]
    MismatchedParentheses,
    #[error("Missing operand during parse.")]
    MissingOperand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserState {
    ExpectOperand,
    ExpectOperator,
}

enum OperableToken {
    Operator(OperatorToken),

    Function { name: String },

    LeftParen { is_function_call: bool, argc: usize },
}

#[derive(Debug, PartialEq)]
enum PostfixToken {
    Operand(Token),
    Operator(OperatorToken),

    Function { name: String, argc: usize },
}

#[derive(Debug, PartialEq)]
struct PostfixExpression(VecDeque<PostfixToken>);

fn shunting_yard(
    tokens: &mut impl Iterator<Item = Token>,
) -> Result<PostfixExpression, ParseError> {
    let mut tokens = tokens.peekable();

    let mut output = VecDeque::new();
    let mut operator_stack: Vec<OperableToken> = Vec::new();

    let mut state = ParserState::ExpectOperand;

    while let Some(token) = tokens.next() {
        match token {
            //
            // ---------------------------------------------------------
            // Number
            // ---------------------------------------------------------
            //
            Token::Number(_) => {
                if state != ParserState::ExpectOperand {
                    return Err(ParseError::UnexpectedToken(token.to_string()));
                }

                output.push_back(PostfixToken::Operand(token));
                state = ParserState::ExpectOperator;
            }

            //
            // ---------------------------------------------------------
            // Identifier
            // ---------------------------------------------------------
            //
            Token::Identifier(name) => {
                if state != ParserState::ExpectOperand {
                    return Err(ParseError::UnexpectedToken(name));
                }

                let is_function = matches!(
                    tokens.peek(),
                    Some(Token::Symbol(SymbolToken::Paren(Parenthesis::Left)))
                );

                if is_function {
                    operator_stack.push(OperableToken::Function { name });
                } else {
                    output.push_back(PostfixToken::Operand(Token::Identifier(name)));

                    state = ParserState::ExpectOperator;
                }
            }

            //
            // ---------------------------------------------------------
            // Operator
            // ---------------------------------------------------------
            //
            Token::Symbol(SymbolToken::Operator(op)) => {
                let op = match state {
                    //
                    // +x or -x
                    //
                    ParserState::ExpectOperand => match op {
                        OperatorToken::Plus => OperatorToken::UnaryPlus,

                        OperatorToken::Minus => OperatorToken::UnaryMinus,

                        _ => {
                            return Err(ParseError::UnexpectedToken(op.to_string()));
                        }
                    },

                    //
                    // x + y or x - y
                    //
                    ParserState::ExpectOperator => op,
                };

                //
                // Pop operators according to precedence and
                // associativity.
                //
                while let Some(top) = operator_stack.last() {
                    let should_pop = match top {
                        OperableToken::Operator(top_op) => top_op.should_pop_before(&op),

                        // Functions and parentheses form barriers.
                        OperableToken::Function { .. } | OperableToken::LeftParen { .. } => false,
                    };

                    if !should_pop {
                        break;
                    }

                    match operator_stack.pop().unwrap() {
                        OperableToken::Operator(top_op) => {
                            output.push_back(PostfixToken::Operator(top_op));
                        }

                        _ => unreachable!(),
                    }
                }

                operator_stack.push(OperableToken::Operator(op));

                state = ParserState::ExpectOperand;
            }

            //
            // ---------------------------------------------------------
            // Left parenthesis
            // ---------------------------------------------------------
            //
            Token::Symbol(SymbolToken::Paren(Parenthesis::Left)) => {
                if state != ParserState::ExpectOperand {
                    return Err(ParseError::UnexpectedToken(token.to_string()));
                }

                let is_function_call =
                    matches!(operator_stack.last(), Some(OperableToken::Function { .. }));

                operator_stack.push(OperableToken::LeftParen {
                    is_function_call,
                    argc: 0,
                });
            }

            //
            // ---------------------------------------------------------
            // Comma
            // ---------------------------------------------------------
            //
            Token::Symbol(SymbolToken::Comma) => {
                //
                // A comma must follow an expression.
                //
                if state != ParserState::ExpectOperator {
                    return Err(ParseError::UnexpectedComma);
                }

                //
                // Pop operators belonging to this argument.
                //
                loop {
                    match operator_stack.last() {
                        Some(OperableToken::Operator(_)) => match operator_stack.pop().unwrap() {
                            OperableToken::Operator(op) => {
                                output.push_back(PostfixToken::Operator(op));
                            }

                            _ => unreachable!(),
                        },

                        Some(OperableToken::LeftParen { .. }) => {
                            break;
                        }

                        Some(OperableToken::Function { .. }) | None => {
                            return Err(ParseError::UnexpectedComma);
                        }
                    }
                }

                //
                // The comma must be inside a function call.
                //
                match operator_stack.last_mut() {
                    Some(OperableToken::LeftParen {
                        is_function_call: true,
                        argc,
                    }) => {
                        *argc += 1;
                    }

                    Some(OperableToken::LeftParen {
                        is_function_call: false,
                        ..
                    }) => {
                        return Err(ParseError::UnexpectedComma);
                    }

                    _ => {
                        return Err(ParseError::UnexpectedComma);
                    }
                }

                state = ParserState::ExpectOperand;
            }

            //
            // ---------------------------------------------------------
            // Right parenthesis
            // ---------------------------------------------------------
            //
            Token::Symbol(SymbolToken::Paren(Parenthesis::Right)) => {
                //
                // Pop operators until '('.
                //
                loop {
                    match operator_stack.last() {
                        Some(OperableToken::Operator(_)) => match operator_stack.pop().unwrap() {
                            OperableToken::Operator(op) => {
                                output.push_back(PostfixToken::Operator(op));
                            }

                            _ => unreachable!(),
                        },

                        Some(OperableToken::LeftParen { .. }) => {
                            break;
                        }

                        Some(OperableToken::Function { .. }) | None => {
                            return Err(ParseError::MismatchedParentheses);
                        }
                    }
                }

                //
                // Remove '('.
                //
                let left_paren = operator_stack.pop().unwrap();

                let (is_function_call, mut argc) = match left_paren {
                    OperableToken::LeftParen {
                        is_function_call,
                        argc,
                    } => (is_function_call, argc),

                    _ => unreachable!(),
                };

                if is_function_call {
                    //
                    // f() is invalid.
                    //
                    if state == ParserState::ExpectOperand && argc == 0 {
                        return Err(ParseError::UnexpectedToken(String::from(")")));
                    }

                    //
                    // f(1,2)
                    //
                    // The final argument wasn't followed by a comma,
                    // so count it here.
                    //
                    if state == ParserState::ExpectOperator {
                        argc += 1;
                    }

                    //
                    // f(1,)
                    //
                    // In ExpectOperand with argc > 0,
                    // this is the allowed trailing comma.
                    //

                    match operator_stack.pop() {
                        Some(OperableToken::Function { name }) => {
                            output.push_back(PostfixToken::Function { name, argc });
                        }

                        _ => {
                            return Err(ParseError::MismatchedParentheses);
                        }
                    }
                } else {
                    //
                    // A normal '(' must contain an expression.
                    //
                    if state != ParserState::ExpectOperator {
                        return Err(ParseError::UnexpectedToken(String::from(")")));
                    }
                }

                state = ParserState::ExpectOperator;
            }
        }
    }

    //
    // An expression cannot end while expecting an operand.
    //
    if state == ParserState::ExpectOperand {
        return Err(ParseError::UnexpectedEndOfExpr);
    }

    //
    // Drain remaining operators.
    //
    while let Some(token) = operator_stack.pop() {
        match token {
            OperableToken::Operator(op) => {
                output.push_back(PostfixToken::Operator(op));
            }

            OperableToken::Function { .. } | OperableToken::LeftParen { .. } => {
                return Err(ParseError::MismatchedParentheses);
            }
        }
    }

    Ok(PostfixExpression(output))
}

pub fn parse(raw: &String) -> Result<Expr, ParseError> {
    parse_from_tokens(&mut tokenize(raw)?)
}

pub(crate) fn parse_from_tokens(
    tokens: &mut impl Iterator<Item = Token>,
) -> Result<Expr, ParseError> {
    let mut postfix_tokens = shunting_yard(tokens)?.0;
    let mut stack: Vec<Expr> = Vec::new();

    while let Some(token) = postfix_tokens.pop_front() {
        match token {
            PostfixToken::Operand(token) => match token {
                Token::Number(num) => stack.push(Expr::from(&num)),
                Token::Identifier(ident) => stack.push(Expr::from(ident)),
                Token::Symbol(_) => return Err(ParseError::UnexpectedToken(token.to_string())),
            },
            PostfixToken::Operator(op) => match op {
                OperatorToken::Plus => apply_add(&mut stack)?,
                OperatorToken::Minus => apply_minus(&mut stack)?,
                OperatorToken::Star => apply_mul(&mut stack)?,
                OperatorToken::Slash => apply_divide(&mut stack)?,
                OperatorToken::Caret => apply_pow(&mut stack)?,
                OperatorToken::UnaryPlus => {}
                OperatorToken::UnaryMinus => apply_negate(&mut stack)?,
            },
            PostfixToken::Function { name, argc } => apply_func(&mut stack, name, argc)?,
        }
    }
    Ok(stack.pop().ok_or(ParseError::MissingOperand)?)
}

fn apply_negate(stack: &mut Vec<Expr>) -> Result<(), ParseError> {
    let expr = Expr::Neg(Box::new(stack.pop().ok_or(ParseError::MissingOperand)?));
    stack.push(expr);
    Ok(())
}

fn apply_add(stack: &mut Vec<Expr>) -> Result<(), ParseError> {
    let rhs = stack.pop().ok_or(ParseError::MissingOperand)?;
    let lhs = stack.pop().ok_or(ParseError::MissingOperand)?;
    match (lhs, rhs) {
        (Expr::Add(mut lhs), Expr::Add(rhs)) => {
            lhs.extend(rhs);
            stack.push(Expr::Add(lhs));
        }
        (Expr::Add(mut lhs), rhs) => {
            lhs.push(rhs);
            stack.push(Expr::Add(lhs))
        }
        (lhs, Expr::Add(mut rhs)) => {
            rhs.push(lhs);
            stack.push(Expr::Add(rhs));
        }
        (lhs, rhs) => stack.push(Expr::Add(vec![lhs, rhs])),
    };
    Ok(())
}

fn apply_minus(stack: &mut Vec<Expr>) -> Result<(), ParseError> {
    apply_negate(stack)?;
    apply_add(stack)
}

fn apply_mul(stack: &mut Vec<Expr>) -> Result<(), ParseError> {
    let rhs = stack.pop().ok_or(ParseError::MissingOperand)?;
    let lhs = stack.pop().ok_or(ParseError::MissingOperand)?;
    match (lhs, rhs) {
        (Expr::Mul(mut lhs), Expr::Mul(rhs)) => {
            lhs.extend(rhs);
            stack.push(Expr::Mul(lhs));
        }
        (Expr::Mul(mut lhs), rhs) => {
            lhs.push(rhs);
            stack.push(Expr::Mul(lhs))
        }
        (lhs, Expr::Mul(mut rhs)) => {
            rhs.push(lhs);
            stack.push(Expr::Mul(rhs));
        }
        (lhs, rhs) => stack.push(Expr::Mul(vec![lhs, rhs])),
    };
    Ok(())
}

fn apply_pow(stack: &mut Vec<Expr>) -> Result<(), ParseError> {
    let rhs = stack.pop().ok_or(ParseError::MissingOperand)?;
    let lhs = stack.pop().ok_or(ParseError::MissingOperand)?;

    stack.push(Expr::Pow(Box::new(lhs), Box::new(rhs)));
    Ok(())
}

fn apply_reciprocate(stack: &mut Vec<Expr>) -> Result<(), ParseError> {
    let expr = Box::new(stack.pop().ok_or(ParseError::MissingOperand)?);
    stack.push(Expr::Pow(
        expr,
        Box::new(Expr::Neg(Box::new(Expr::Integer(1)))),
    ));
    Ok(())
}

fn apply_divide(stack: &mut Vec<Expr>) -> Result<(), ParseError> {
    apply_reciprocate(stack)?;
    apply_mul(stack)
}

fn apply_func(stack: &mut Vec<Expr>, name: String, arity: usize) -> Result<(), ParseError> {
    let args = stack.split_off(stack.len().saturating_sub(arity));
    stack.push(Expr::Call {
        function: Symbol::create(name),
        args,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::parser::{lexer::OperatorToken::*, numerical_string::NumericalString};

    use super::*;
    use rstest::*;

    fn int(n: i64) -> PostfixToken {
        PostfixToken::Operand(Token::Number(NumericalString::from(n)))
    }
    fn real(n: f64) -> PostfixToken {
        PostfixToken::Operand(Token::Number(NumericalString::from(n)))
    }
    fn iden(s: &str) -> PostfixToken {
        PostfixToken::Operand(Token::Identifier(s.to_string()))
    }
    fn op(op: OperatorToken) -> PostfixToken {
        PostfixToken::Operator(op)
    }
    fn func(s: &str, argc: usize) -> PostfixToken {
        PostfixToken::Function {
            name: s.to_string(),
            argc,
        }
    }

    #[rstest]
    #[case("2+2", vec![int(2), int(2), op(Plus)])]
    #[case("2*3", vec![int(2), int(3), op(Star)])]
    #[case("3.1415 * 5 - 2", vec![real(3.1415), int(5), op(Star), int(2), op(Minus)])]
    #[case("3.1415 * r^2", vec![real(3.1415), iden("r"), int(2), op(Caret), op(Star)])]
    #[case("f(1)", vec![int(1), func("f", 1)])]
    #[case("f(1 + 2)", vec![
        int(1),
        int(2),
        op(Plus),
        func("f", 1)
    ])]
    #[case("f(g(1))", vec![
        int(1),
        func("g", 1),
        func("f", 1)
    ])]
    #[case("f(1, 2)", vec![
        int(1),
        int(2),
        func("f", 2),
    ])]
    #[case("f(1 + 2, 3 * 4)", vec![
        int(1),
        int(2),
        op(Plus),
        int(3),
        int(4),
        op(Star),
        func("f", 2),
    ])]
    #[case("2 * f(1)", vec![
        int(2),
        int(1),
        func("f", 1),
        op(Star),
    ])]
    #[case("1", vec![int(1)])]
    #[case("(1)", vec![int(1)])]
    #[case("((1))", vec![int(1)])]
    #[case("f(1)", vec![int(1), func("f", 1)])]
    #[case("f((1))", vec![int(1), func("f", 1)])]
    #[case("f(1,)", vec![int(1), func("f", 1)])]
    #[case("f(g(1))", vec![
        int(1),
        func("g", 1),
        func("f", 1),
    ])]
    #[case("1 + 2 * 3", vec![
        int(1),
        int(2),
        int(3),
        op(Star),
        op(Plus),
    ])]
    #[case("(1 + 2) * 3", vec![
        int(1),
        int(2),
        op(Plus),
        int(3),
        op(Star),
    ])]
    #[case("r", vec![iden("r")])]
    #[case("f(x)", vec![iden("x"), func("f", 1)])]
    #[case("sin(pi)", vec![iden("pi"), func("sin", 1)])]
    #[case("sin(0)", vec![int(0), func("sin", 1)])]
    #[case("-1", vec![int(1), op(UnaryMinus)])]
    #[case("--1", vec![int(1), op(UnaryMinus), op(UnaryMinus)])]
    #[case("+1", vec![int(1), op(UnaryPlus)])]
    #[case("-2^2", vec![int(2), int(2), op(Caret), op(UnaryMinus)])]
    #[case("(-2)^2", vec![int(2), op(UnaryMinus), int(2), op(Caret)])]
    #[case("2^(-3)", vec![int(2), int(3), op(UnaryMinus), op(Caret)])]
    fn test_shunting_yard(#[case] raw: &str, #[case] expected: Vec<PostfixToken>) {
        assert_eq!(
            PostfixExpression(VecDeque::from(expected)),
            shunting_yard(&mut tokenize(&raw.to_string()).unwrap()).unwrap()
        )
    }

    #[rstest]
    #[case("1+")]
    #[case("1^")]
    #[case("r^")]
    #[case("f(")]
    #[case("f(,,)")]
    #[case("20 / 2 +")]
    #[case(",,")]
    #[case("x,y")]
    fn test_invalid_expr(#[case] raw: &str) {
        assert!(shunting_yard(&mut tokenize(&raw.to_string()).unwrap()).is_err())
    }

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
            function: Symbol::from(ident),
            args: args.to_vec(),
        }
    }

    #[rstest]
    #[case("1", e_int(1))]
    #[case("-1", neg(e_int(1)))]
    #[case("(1)", e_int(1))]
    #[case("(-1)", neg(e_int(1)))]
    #[case("((1))", e_int(1))]
    #[case("1+1", add(&[e_int(1), e_int(1)]))]
    #[case("1+2+3", add(&[e_int(1), e_int(2), e_int(3)]))]
    #[case("1-2+3", add(&[e_int(1), neg(e_int(2)), e_int(3)]))]
    #[case("2*5", mul(&[e_int(2), e_int(5)]))]
    #[case("2/5", mul(&[e_int(2), reciprocal(e_int(5))]))]
    #[case("sin(x)", e_func("sin", &[e_iden("x")]))]
    #[case("3.14159", e_real(3.14159))]
    #[case("pi^2", pow(e_iden("pi"), e_int(2)))]
    #[case("-2^2", neg(pow(e_int(2), e_int(2))))]
    #[case("a^b * a^c", mul(&[pow(e_iden("a"), e_iden("b")), pow(e_iden("a"), e_iden("c"))]))]
    #[case("a^(b+c)", pow(e_iden("a"), add(&[e_iden("b"), e_iden("c")])))]
    #[case("0.5 - 0.5", add(&[e_real(0.5), neg(e_real(0.5))]))]
    #[case("1 / 2 * 4", mul(&[e_int(1), reciprocal(e_int(2)), e_int(4)]))]
    #[case("f(1+2, 3+4)", e_func("f", &[add(&[e_int(1), e_int(2)]), add(&[e_int(3), e_int(4)])]))]
    #[case("f(x, y)", e_func("f", &[e_iden("x"), e_iden("y")]))]
    fn test_parse(#[case] raw: &str, #[case] expected: Expr) {
        assert_eq!(expected, parse(&raw.to_string()).unwrap())
    }
}

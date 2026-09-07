/*
The parser should take as input an iterable collection of tokens and return the
abstract syntax tree as a valid math expression if the tokens represent a valid expression.
*/

use thiserror::Error;

use crate::{mathematics::expression::Expr, parser::lexer::*};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to tokenize expression: {0}")]
    TokenizationError(#[from] TokenizationError),
    #[error("Unexpected Operator: {0}.")]
    UnexpectedToken(Token),
    #[error("Unexpected end of expression.")]
    UnexpectedEndOfExpr,
    #[error("Unexpected comma.")]
    UnexpectedComma,
    #[error("Mismatched Parentheses.")]
    MismatchedParentheses,
}

pub(crate) fn parse_from_tokens(tokens: impl Iterator<Item = Token>) -> Result<Expr, ParseError> {
    todo!()
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
struct PostfixExpression(Vec<PostfixToken>);

fn shunting_yard(
    tokens: &mut impl Iterator<Item = Token>,
) -> Result<PostfixExpression, ParseError> {
    let mut tokens = tokens.peekable();

    let mut output = Vec::new();
    let mut operator_stack: Vec<OperableToken> = Vec::new();

    let mut state = ParserState::ExpectOperand;

    while let Some(token) = tokens.next() {
        match token {
            // Operands
            Token::Number(_) => {
                if state != ParserState::ExpectOperand {
                    return Err(ParseError::UnexpectedToken(token));
                }

                output.push(PostfixToken::Operand(token));
                state = ParserState::ExpectOperator;
            }

            Token::Identifier(name) => {
                if state != ParserState::ExpectOperand {
                    return Err(ParseError::UnexpectedToken(Token::Identifier(name)));
                }

                let is_function = matches!(
                    tokens.peek(),
                    Some(Token::Symbol(SymbolToken::Paren(Parenthesis::Left)))
                );

                if is_function {
                    operator_stack.push(OperableToken::Function { name });

                    // The following '(' will establish the function
                    // argument frame.
                } else {
                    output.push(PostfixToken::Operand(Token::Identifier(name)));

                    state = ParserState::ExpectOperator;
                }
            }

            // Operators
            Token::Symbol(SymbolToken::Operator(op)) => {
                if state != ParserState::ExpectOperator {
                    return Err(ParseError::UnexpectedToken(op.into()));
                }

                while let Some(top) = operator_stack.last() {
                    let should_pop = match top {
                        OperableToken::Operator(top_op) => top_op.should_pop_before(&op),

                        OperableToken::Function { .. } | OperableToken::LeftParen { .. } => false,
                    };

                    if !should_pop {
                        break;
                    }

                    match operator_stack.pop().unwrap() {
                        OperableToken::Operator(top_op) => {
                            output.push(PostfixToken::Operator(top_op));
                        }

                        _ => unreachable!(),
                    }
                }

                operator_stack.push(OperableToken::Operator(op));

                state = ParserState::ExpectOperand;
            }

            // Left parenthesis
            Token::Symbol(SymbolToken::Paren(Parenthesis::Left)) => {
                if state != ParserState::ExpectOperand {
                    return Err(ParseError::UnexpectedToken(token));
                }

                let is_function_call =
                    matches!(operator_stack.last(), Some(OperableToken::Function { .. }));

                operator_stack.push(OperableToken::LeftParen {
                    is_function_call,
                    argc: 0,
                });
            }

            // Comma
            Token::Symbol(SymbolToken::Comma) => {
                // A comma must follow a complete argument.
                if state != ParserState::ExpectOperator {
                    return Err(ParseError::UnexpectedComma);
                }

                // Pop operators belonging to the current argument.
                loop {
                    match operator_stack.last() {
                        Some(OperableToken::Operator(_)) => match operator_stack.pop().unwrap() {
                            OperableToken::Operator(op) => {
                                output.push(PostfixToken::Operator(op));
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

                // The comma must belong to a function call.
                match operator_stack.last_mut() {
                    Some(OperableToken::LeftParen {
                        is_function_call: true,
                        argc: argument_count,
                    }) => {
                        *argument_count += 1;
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

                // A new argument must follow the comma.
                state = ParserState::ExpectOperand;
            }

            // Right parenthesis
            Token::Symbol(SymbolToken::Paren(Parenthesis::Right)) => {
                // First, pop operators until we reach '('.
                loop {
                    match operator_stack.last() {
                        Some(OperableToken::Operator(_)) => match operator_stack.pop().unwrap() {
                            OperableToken::Operator(op) => {
                                output.push(PostfixToken::Operator(op));
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

                // Now the top of the stack must be '('.
                let left_paren = operator_stack.pop().unwrap();

                let (is_function_call, mut argc) = match left_paren {
                    OperableToken::LeftParen {
                        is_function_call,
                        argc,
                    } => (is_function_call, argc),

                    _ => unreachable!(),
                };

                // If this is a function call, validate the argument state
                // and resolve its final argument count.
                if is_function_call {
                    match state {
                        //
                        // f(1)
                        // f(1, 2)
                        //
                        ParserState::ExpectOperator => {
                            argc += 1;
                        }

                        //
                        // f(1,)
                        // f(1,2,)
                        //
                        ParserState::ExpectOperand if argc > 0 => {
                            // This is a valid trailing comma.
                        }

                        //
                        // f()
                        // f(,)
                        //
                        ParserState::ExpectOperand => {
                            return Err(ParseError::UnexpectedToken(Token::Symbol(
                                SymbolToken::Paren(Parenthesis::Right),
                            )));
                        }
                    }

                    //
                    // The Function must immediately precede its '('.
                    //
                    match operator_stack.pop() {
                        Some(OperableToken::Function { name }) => {
                            output.push(PostfixToken::Function { name, argc });
                        }

                        _ => {
                            return Err(ParseError::MismatchedParentheses);
                        }
                    }
                } else {
                    //
                    // A normal grouping parenthesis cannot be closed while
                    // we're still expecting an operand.
                    //
                    if state != ParserState::ExpectOperator {
                        return Err(ParseError::UnexpectedToken(Token::Symbol(
                            SymbolToken::Paren(Parenthesis::Right),
                        )));
                    }
                }

                state = ParserState::ExpectOperator;
            }
        }
    }

    // An expression cannot end while expecting an operand.
    if state == ParserState::ExpectOperand {
        return Err(ParseError::UnexpectedEndOfExpr);
    }

    // Drain the remaining operators.
    while let Some(token) = operator_stack.pop() {
        match token {
            OperableToken::Operator(op) => {
                output.push(PostfixToken::Operator(op));
            }

            OperableToken::Function { .. } | OperableToken::LeftParen { .. } => {
                return Err(ParseError::MismatchedParentheses);
            }
        }
    }

    Ok(PostfixExpression(output))
}

pub fn parse(raw: &String) -> Result<Expr, ParseError> {
    parse_from_tokens(tokenize(raw)?)
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
    fn test_shunting_yard(#[case] raw: &str, #[case] expected: Vec<PostfixToken>) {
        assert_eq!(
            PostfixExpression(expected),
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
}

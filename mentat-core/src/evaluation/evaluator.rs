use thiserror::Error;

use crate::{
    evaluation::environment::EvalEnv,
    mathematics::{expression::Expr, function::FunctionError, symbol::Symbol},
};

pub struct Evaluator<'env> {
    env: &'env EvalEnv<'env>,
}

#[derive(Debug, Error)]
pub enum EvaluationError {
    #[error("Found an empty expression where one was expected.")]
    EmptyExpression,
    #[error("Error occured with symbol '{0}'. {1}")]
    FunctionError(Symbol, FunctionError),
    #[error("Found an unknown symbol, '{0}'. Please define '{0}' to use it in expressions.")]
    UnknownSymbol(Symbol),
    #[error("The function '{symbol}' expects {expected} arguments but {found} was given.")]
    BadArity {
        symbol: Symbol,
        expected: usize,
        found: usize,
    },
}

impl<'env> Evaluator<'env> {
    pub fn new(env: &'env EvalEnv) -> Self {
        Self { env }
    }

    pub fn evaluate(&self, expr: Expr) -> Result<Expr, EvaluationError> {
        match expr {
            Expr::Integer(_) => Ok(expr),
            Expr::Real(_) => Ok(expr),
            Expr::Add(exprs) => {
                let mut exprs = exprs.into_iter();
                let first = exprs.next().ok_or(EvaluationError::EmptyExpression)?;
                exprs.try_fold(self.evaluate(first)?, |acc, expr| {
                    Ok(acc + self.evaluate(expr)?)
                })
            }
            Expr::Mul(exprs) => {
                let mut exprs = exprs.into_iter();
                let first = exprs.next().ok_or(EvaluationError::EmptyExpression)?;
                exprs.try_fold(self.evaluate(first)?, |acc, expr| {
                    Ok(acc * self.evaluate(expr)?)
                })
            }

            Expr::Pow(expr, expr1) => Ok(self.evaluate(*expr)?.pow(self.evaluate(*expr1)?)),
            Expr::Neg(expr) => Ok(Expr::Neg(Box::new(self.evaluate(*expr)?))),
            // function needs to be refactored to be a SYMBOL,
            Expr::Call { function, args } => {
                // Create an extended EvalEnv with the variable definitions
                // evaluate the function's expression within that environmnet
                // return the resulting expression

                // Handle the atomic case where x is just a number

                let func = self
                    .env
                    .get(&function)
                    .ok_or_else(|| EvaluationError::UnknownSymbol(function.clone()))?;

                let env = func
                    .create_env(args, self.env)
                    .map_err(|err| EvaluationError::FunctionError(function.clone(), err))?;
                let evaluator = Evaluator::new(&env);

                Ok(evaluator.evaluate(func.expr.clone())?)
            }
        }
    }
}
mod tests {
    use rstest::*;

    #[rstest]
    #[case("2", "2")]
    #[case("2*5", "10")]
    #[case("0.5 * 0.5", "0.25")]
    #[case("100+10000", "10100")]
    #[case("100.0+10000", "10100.0")]
    #[case("2^3", "8")]
    #[case("2.0^3", "8.0")]
    fn test_evaluate(#[case] expr: &str, #[case] expected: &str) {
        use crate::{
            evaluation::{environment::EvalEnv, evaluator::Evaluator},
            parser::parser::parse,
        };

        let expr = parse(&expr.to_string()).expect("Failed to parse expression");
        let expected = parse(&expected.to_string()).expect("Failed to parse expected");

        let env = EvalEnv::new();
        let evaluator = Evaluator::new(&env);

        assert_eq!(
            expected,
            evaluator
                .evaluate(expr)
                .expect("Failed to evaluate expression")
        );
    }

    #[rstest]
    #[case("f(2)", "4")]
    #[case("f(5)", "7")]
    fn test_evaluate_functions(#[case] expr: &str, #[case] expected: &str) {
        use crate::{
            evaluation::{environment::EvalEnv, evaluator::Evaluator},
            mathematics::symbol::Symbol,
            parser::parser::parse,
        };

        let expr = parse(&expr.to_string()).expect("Failed to parse expression");
        let expected = parse(&expected.to_string()).expect("Failed to parse expected");

        let mut env = EvalEnv::new();
        let sym = Symbol::from("f");
        env.set(
            &sym,
            parse(&String::from("x+2")).expect("Failed to parse function f"),
        );
        let evaluator = Evaluator::new(&env);

        assert_eq!(
            expected,
            evaluator
                .evaluate(expr)
                .expect("Failed to evaluate expression")
        );
    }
}

use thiserror::Error;

use crate::{
    evaluation::environment::EvalEnv,
    mathematics::{expression::Expr, symbol::Symbol},
};

#[derive(Debug)]
pub struct Function {
    pub expr: Expr,
    parameters: Vec<Symbol>,
}

#[derive(Debug, Error)]
pub enum FunctionError {
    #[error("The function expected {expected} arguments but {found} were found.")]
    BadArity { expected: usize, found: usize },
}

impl From<Expr> for Function {
    fn from(expr: Expr) -> Self {
        let parameters = expr.symbols();
        Self { expr, parameters }
    }
}

impl<'env> Function {
    pub fn create_env(
        &'env self,
        arguments: Vec<Expr>,
        env: &'env EvalEnv,
    ) -> Result<EvalEnv<'env>, FunctionError> {
        let args_len = arguments.len();
        let arity = self.arity();
        if arity != args_len {
            return Err(FunctionError::BadArity {
                expected: arity,
                found: args_len,
            });
        }
        let mut env = env.extend();
        arguments.into_iter().enumerate().for_each(|(i, arg)| {
            env.set(&self.parameters[i], arg);
        });
        Ok(env)
    }
}

impl Function {
    pub fn arity(&self) -> usize {
        self.parameters.len()
    }
}

use std::collections::HashMap;

use crate::mathematics::{expression::Expr, function::Function, symbol::Symbol};

/**
An EvalulationEnvironment (EvalEnv) represents a set of definitions required to evaluate an expression
*/
#[derive(Debug)]
pub struct EvalEnv<'env> {
    definitions: HashMap<&'env Symbol, Function>,
    parent: Option<&'env EvalEnv<'env>>,
}

impl EvalEnv<'_> {
    pub fn new() -> Self {
        EvalEnv {
            definitions: HashMap::new(),
            parent: None,
        }
    }
}

impl<'env> EvalEnv<'env> {
    pub fn set(&mut self, symbol: &'env Symbol, expr: Expr) {
        self.definitions.insert(symbol, expr.into());
    }

    pub fn set_fn(&mut self, symbol: &'env Symbol, function: Function) {
        self.definitions.insert(symbol, function);
    }

    pub fn get(&self, symbol: &Symbol) -> Option<&Function> {
        if let Some(val) = self.definitions.get(symbol) {
            return Some(val);
        }
        if let Some(parent) = self.parent {
            return parent.get(symbol);
        }
        return None;
    }

    pub fn extend(&'env self) -> EvalEnv<'env> {
        EvalEnv {
            definitions: HashMap::new(),
            parent: Some(self),
        }
    }
}

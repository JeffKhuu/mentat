/*
The normalizer should take as input a valid expression and return an expression in canonicalized form.

Transformations should be determinant and idempotent

An expression in canonicalized form will follow these rules:
- All constant values must be combined into a single value (Real or integer)
- All constant values will appear as the last term of commutative operations
- All function values will appear in sorted order (by the partial ordering defined on Symbol) on commutative operations
- Identity operations will be simplified
*/

/*
TODO:
- We can use a zero-sized type abstraction to ensure our expressions are normalized in the type system (See PhantomData<State> pattern)
- These NormalizationRules are complicated on flattened expressions, we need a number of helper functions in Expr to keep the code here straight forward
- We should define a number of basic operations on expressions (+, -, /, *, ^) so that the logic and assumptions we make on them are centralized (i.e what does Expr + Expr look like for all types of Expr?)
*/

use crate::mathematics::expression::Expr;

pub trait NormalizationRule {
    /**
    apply will apply the normalization rule to the given expression.
    */
    fn apply(&self, expr: Expr) -> Expr;
}

pub struct SortedRule;
impl NormalizationRule for SortedRule {
    fn apply(&self, expr: Expr) -> Expr {
        match expr {
            Expr::Add(exprs) => Expr::Add(sort_callable(exprs)),
            Expr::Mul(exprs) => Expr::Mul(sort_callable(exprs)),
            _ => expr,
        }
    }
}

fn sort_callable(mut exprs: Vec<Expr>) -> Vec<Expr> {
    let mut symbols: Vec<Expr> = exprs
        .extract_if(.., |expr| {
            matches!(
                expr,
                Expr::Call {
                    function: _,
                    args: _
                }
            )
        })
        .collect();
    symbols.sort_by(|a, b| {
        let Expr::Call {
            function: function_a,
            args: _,
        } = a
        else {
            unreachable!();
        };
        let Expr::Call {
            function: function_b,
            args: _,
        } = b
        else {
            unreachable!();
        };
        function_a.cmp(&function_b)
    });
    symbols.append(&mut exprs);
    symbols
}

pub struct AdditiveIdentityRule;
impl NormalizationRule for AdditiveIdentityRule {
    fn apply(&self, expr: Expr) -> Expr {
        if let Expr::Add(exprs) = expr {
            let exprs: Vec<Expr> = exprs
                .into_iter()
                .filter(|expr| match expr {
                    Expr::Integer(0) => false,
                    Expr::Real(0.0) => false,
                    _ => true,
                })
                .collect();
            if exprs.len() == 0 {
                return Expr::Integer(0);
            }
            if exprs.len() == 1 {
                return exprs.into_iter().next().unwrap();
            }
            Expr::Add(exprs)
        } else {
            expr
        }
    }
}

pub struct MultiplicativeIdentityRule;
impl NormalizationRule for MultiplicativeIdentityRule {
    fn apply(&self, expr: Expr) -> Expr {
        if let Expr::Mul(exprs) = expr {
            let exprs: Vec<Expr> = exprs
                .into_iter()
                .filter(|expr| match expr {
                    Expr::Integer(1) => false,
                    Expr::Real(1.0) => false,
                    _ => true,
                })
                .collect();
            if exprs.len() == 0 {
                return Expr::Integer(1);
            }
            if exprs.len() == 1 {
                return exprs.into_iter().next().unwrap();
            }
            return Expr::Mul(exprs);
        }
        expr
    }
}

pub struct ConstantFoldRule;
impl NormalizationRule for ConstantFoldRule {
    fn apply(&self, expr: Expr) -> Expr {
        match expr {
            Expr::Add(mut exprs) => {
                // TODO: This function is ugly, clean it up
                let mut int_sum = 0;
                let mut int_neg = 0;
                let mut real_sum = 0.0;
                let size = exprs.len();
                exprs
                    .extract_if(.., |expr| {
                        matches!(expr, Expr::Integer(_) | Expr::Real(_) | Expr::Neg(_))
                    })
                    .for_each(|expr| match expr {
                        Expr::Integer(n) => int_sum += n,

                        Expr::Neg(n) => match *n {
                            Expr::Integer(n) => int_neg += n,
                            Expr::Real(n) => real_sum -= n,
                            _ => {}
                        },
                        Expr::Real(n) => real_sum += n,
                        _ => unreachable!(),
                    });

                let expr = if real_sum == 0.0 {
                    if int_sum > int_neg {
                        int_sum = int_sum - int_neg;
                        Expr::Integer(int_sum)
                    } else {
                        int_sum = int_neg - int_sum;
                        Expr::Neg(Box::new(Expr::Integer(int_sum)))
                    }
                } else {
                    real_sum += int_sum as f64;
                    real_sum -= int_neg as f64;
                    if real_sum > 0.0 {
                        Expr::Real(real_sum)
                    } else {
                        Expr::Neg(Box::new(Expr::Real(-real_sum)))
                    }
                };

                if exprs.len() == size {
                    // no changes were made
                    return Expr::Add(exprs);
                }

                if exprs.len() == 0 {
                    // The whole expression were literals
                    return expr;
                }
                exprs.push(expr);
                Expr::Add(exprs)
            }
            Expr::Mul(mut exprs) => {
                let mut int_prod = 1;
                let mut neg = false; // false if positive
                let mut real_prod = 1.0;
                let size = exprs.len();
                exprs
                    .extract_if(.., |expr| {
                        matches!(expr, Expr::Integer(_) | Expr::Real(_) | Expr::Neg(_))
                    })
                    .for_each(|expr| match expr {
                        Expr::Integer(n) => int_prod *= n,
                        Expr::Real(n) => real_prod *= n,
                        Expr::Neg(n) => match *n {
                            Expr::Integer(n) => {
                                int_prod *= n;
                                neg = !neg;
                            }
                            Expr::Real(n) => {
                                real_prod *= n;
                                neg = !neg;
                            }
                            _ => {}
                        },
                        _ => unreachable!(),
                    });
                let expr = if real_prod == 1.0 {
                    if neg {
                        Expr::Neg(Box::new(Expr::Integer(int_prod)))
                    } else {
                        Expr::Integer(int_prod)
                    }
                } else {
                    if neg {
                        Expr::Neg(Box::new(Expr::Real(real_prod * int_prod as f64)))
                    } else {
                        Expr::Real(real_prod * int_prod as f64)
                    }
                };

                if exprs.len() == size {
                    // no changes were made
                    return Expr::Mul(exprs);
                }

                if exprs.len() == 0 {
                    return expr;
                }
                exprs.push(expr);
                Expr::Mul(exprs)
            }
            expr => expr,
        }
    }
}

pub struct Normalizer {
    rules: Vec<Box<dyn NormalizationRule>>,
}

impl Normalizer {
    pub fn create(rules: Vec<Box<dyn NormalizationRule>>) -> Normalizer {
        Normalizer { rules }
    }

    pub(crate) fn apply_rules(&self, expr: Expr) -> Expr {
        self.rules.iter().fold(expr, |expr, rule| rule.apply(expr))
    }
    pub(crate) fn apply_rules_vec(&self, exprs: Vec<Expr>) -> Vec<Expr> {
        exprs
            .into_iter()
            .map(|expr| self.apply_rules(expr))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        normalizer::{NormalizationRule, *},
        parser::parse,
    };
    use rstest::*;
    #[rstest]
    #[case("2+1", "3", ConstantFoldRule {})]
    #[case("x+1+2+3", "x+6", ConstantFoldRule {})]
    #[case("x+100-50", "x+50", ConstantFoldRule {})]
    #[case("(x+5)+1+100", "x+106", ConstantFoldRule {})]
    #[case("x*1*2*3*4*5*6", "x*720", ConstantFoldRule {})]
    #[case("r+2.0+2.5+2.75", "r+7.25", ConstantFoldRule {})]
    #[case("r*0.5*0.5", "r*0.25", ConstantFoldRule {})]
    #[case("y-5+2", "y-3", ConstantFoldRule {})]
    #[case("y * -1 * -1", "y * 1", ConstantFoldRule {})]
    #[case("y * 1 * -1", "y * -1", ConstantFoldRule {})]
    #[case("y * 2 * 6.0", "y * 12.0", ConstantFoldRule {})]
    #[case("y * -2 * 6.0", "y * -12.0", ConstantFoldRule {})]
    #[case("y * 2 * -6.0", "y * -12.0", ConstantFoldRule {})]
    #[case("y + (x*2)", "y+(x*2)", ConstantFoldRule {})]
    #[case("x*y", "x*y", ConstantFoldRule {})]
    #[case("y * 0.6 * -1.2", "y * -0.72", ConstantFoldRule {})]
    #[case("x+0+1+2", "x+1+2", AdditiveIdentityRule {})]
    #[case("((x+2) + 0) + 0", "x+2", AdditiveIdentityRule {})]
    #[case("((x*2) + 0) + 0", "x*2", AdditiveIdentityRule {})]
    #[case("((x+2) * 1) * 1", "x+2", MultiplicativeIdentityRule {})]
    #[case("((x*2) * 1) * 1", "x*2", MultiplicativeIdentityRule {})]
    #[case("x+y+z", "x+y+z", SortedRule{})]
    #[case("z+y+x", "x+y+z", SortedRule{})]
    #[case("a+b+15", "a+b+15", SortedRule{})]
    #[case("g+a+20", "a+g+20", SortedRule{})]
    #[case("x+y+5+10", "x+y+5+10", SortedRule{})]
    #[case("b+a+5+10", "a+b+5+10", SortedRule{})]
    fn test_single_rule(
        #[case] expr_str: &str,
        #[case] expected_str: &str,
        #[case] rule: impl NormalizationRule,
    ) {
        let expected = parse(&expected_str.to_string()).unwrap();
        assert_eq!(expected, rule.apply(parse(&expr_str.to_string()).unwrap()))
    }
}

use mentat_core::{
    self,
    evaluation::{environment::EvalEnv, evaluator::Evaluator},
    mathematics::{expression::Expr, symbol::Symbol},
    parser::{
        normalizer::{
            AdditiveIdentityRule, ConstantFoldRule, MultiplicativeIdentityRule, NormalizationRule,
            Normalizer, SortedRule,
        },
        parser::parse,
    },
};
pub(crate) fn evaluate_and_print(line: String) -> () {
    // Evaluate
    let expr: Expr = match parse(&line) {
        Ok(expr) => expr,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let rules: Vec<Box<dyn NormalizationRule>> = vec![
        Box::new(MultiplicativeIdentityRule),
        Box::new(AdditiveIdentityRule),
        Box::new(ConstantFoldRule {}),
        Box::new(SortedRule {}),
    ];
    let normalizer = Normalizer::create(rules);

    let mut env = EvalEnv::new();
    let sym_f: Symbol = Symbol::from("f");
    let expr_f: Expr = parse(&"x^2".to_string()).expect("Failed to parse f");
    env.set(&sym_f, expr_f);
    let evaluator = Evaluator::new(&env);

    let expr = expr.normalize(&normalizer);
    let expr = match evaluator.evaluate(expr) {
        Ok(expr) => expr,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    // Print
    println!("{}", expr);
}

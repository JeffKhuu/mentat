use mentat_core::{
    self,
    evaluation::{environment::EvalEnv, evaluator::Evaluator},
    lang::statement::Statement,
    parser::{
        normalizer::{
            AdditiveIdentityRule, ConstantFoldRule, MultiplicativeIdentityRule, NormalizationRule,
            Normalizer, SortedRule,
        },
        parse,
    },
};
pub(crate) fn evaluate_and_print(line: String) -> () {
    // Evaluate
    let statement: Statement = match parse(&line) {
        Ok(expr) => expr,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };
    let rules: Vec<Box<dyn NormalizationRule>> = vec![
        Box::new(MultiplicativeIdentityRule),
        Box::new(AdditiveIdentityRule),
        Box::new(SortedRule {}),
    ];
    let normalizer = Normalizer::create(rules);

    let env = EvalEnv::new();
    let evaluator = Evaluator::new(&env);

    match statement {
        Statement::Expression(expr) => {
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
        Statement::Assignment(identifier, expr) => todo!(),
    }
}

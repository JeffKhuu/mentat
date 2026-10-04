use mentat_core::{
    self, mathematics::expression::Expr, parser::{
        normalizer::{
            AdditiveIdentityRule, ConstantFoldRule, MultiplicativeIdentityRule, NormalizationRule, Normalizer, SortedRule,
        }, parser::parse,
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
    let expr = expr.normalize(&normalizer);

    // Print
    println!("{}", expr);
}

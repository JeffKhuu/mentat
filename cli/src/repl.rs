use mentat_core::{self, mathematics::expression::Expr, parser::parser::parse};
pub(crate) fn evaluate_and_print(line: String) -> () {
    // Evaluate
    let expr: Expr = match parse(&line) {
        Ok(expr) => expr,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    // TODO: Future Evaluation Logic

    // ...

    // Print
    println!("{}", expr);
}

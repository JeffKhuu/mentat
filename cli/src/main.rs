use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use crate::repl::evaluate_and_print;
mod repl;

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                evaluate_and_print(line);
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("An Error Occurred: {:?}", err);
                break;
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history("history.txt");
    Ok(())
}

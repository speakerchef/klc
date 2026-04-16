mod lexer;
use std::{env::args, error::Error, process::exit};

use crate::lexer::Lexer;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        eprintln!("Error: Please supply arguments: path/to/klc <FILE.knv> <EXEC-NAME>");
        exit(1)
    }

    let path = &args[1];
    let exec_name = &args[2];

    let mut lex = Lexer::new();
    lex.tokenize(path)?;
    Ok(())
}

use std::env;
use flowscript::lexer::Lexer;
use flowscript::reader::{FileReader, Repl};

fn main() {
    let args: Vec<String> = env::args().collect();
    let reader: Box<dyn Iterator<Item = String>> = match args.len() {
        1 => Box::new(Repl),
        2 => Box::new(FileReader::new(&args[1])),
        _ => {
            println!("Usage: fscc [script]");
            std::process::exit(64);
        }
    };
    let mut lexer = Lexer::new(reader);
}

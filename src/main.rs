use std::env;
use flowscript::error_handler::ErrorHandler;
use flowscript::lexer::Lexer;
use flowscript::parser::Parser;
use flowscript::reader::{FileReader, Repl};
use flowscript::optimizer::Optimizer;
use flowscript::symbol_table::SymbolTable;
use flowscript::constants_pool::ConstantsPool;
use flowscript::emitter::Emitter;
use flowscript::virmac::VirMac;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let mut error_handler = ErrorHandler::default();
    match args.len() {
        1 => repl(),
        2 => read_file(args.pop().unwrap()),
        _ => {
            println!("Usage: fscc [script]");
            std::process::exit(64);
        }
    }
}

fn repl() {
    while let Some(Ok(source_code)) = Repl.next() {
        run(source_code)
    }
}

fn read_file(path: String) {
    match FileReader::new(path).next() {
        Some(Ok(source_code)) => run(source_code),
        Some(Err(error)) => todo!(),
        _ => unreachable!()
    }
}

fn run(source_code: Vec<u8>) {
    let lexer = Lexer::new(&source_code);
    let mut parser = Parser::new(lexer);
    let mut optimizer = Optimizer::new(parser);
    let ast = optimizer.optimize();
    let symbol_table = SymbolTable::new();
    let constants_pool = ConstantsPool::default();
    let mut emitter = Emitter::new(symbol_table, constants_pool);
    let (constants_pool, map) = emitter.emit(ast);
    let mut virmac = VirMac::new(constants_pool.constants);
    virmac.execute(map);
}
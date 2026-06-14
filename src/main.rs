use std::env;
use flowscript::error_handler::ErrorHandler;
use flowscript::lexer::Lexer;
use flowscript::parser::Parser;
use flowscript::reader::{FileReader, Repl};
use flowscript::optimizer::Optimizer;
use flowscript::symbol_table::SymbolTable;
use flowscript::resolver::Resolver;
use flowscript::constants_pool::ConstantsPool;
use flowscript::emitter::Emitter;
use flowscript::memory::Memory;
use flowscript::assembler::Assembler;
use flowscript::virmac::VirMac;

fn main() {
    let mut args: Vec<String> = env::args().collect();
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
    let mut error_handler = ErrorHandler::default();
    let memory = Memory::new();
    let lexer = Lexer::new(&source_code);
    let mut parser = Parser::new(lexer);
    let mut optimizer = Optimizer::new(parser, error_handler);
    let (nodes, error_handler) = optimizer.optimize();
    let symbol_table = SymbolTable::new();
    let mut resolver = Resolver::new(error_handler, symbol_table);
    let (error_handler, total_arity, ast) = resolver.resolve(nodes);
    let constants_pool = ConstantsPool::default();
    let mut emitter = Emitter::new(constants_pool);
    let (constants_pool, map) = emitter.emit(ast);
    let mut asm = Assembler::new(memory, constants_pool);
    let (byte_map, starts, constants, memory) = asm.assemble_map(map);
    let mut virmac = VirMac::new(memory, error_handler, constants, starts);
    virmac.execute(byte_map, total_arity);
}
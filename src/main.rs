mod lexer;
mod token;
mod node;
mod bytecode;
mod reader;

use reader::Reader;
use crate::lexer::Lexer;
use crate::token::{Token, TokenType::EOF};

fn main() {
    let path = "main.fscc";
    let reader = Reader::new(path);
    let lexer = Lexer::new(reader);
}

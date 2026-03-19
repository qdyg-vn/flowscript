use crate::instructions::{Instruction, Chunk};
use crate::node::Node;
use crate::error_handler::Error;
use crate::symbol_table::{SymbolTable, SymbolType};

pub struct Emitter {
    errors: Vec<Error>,
    symbol_table: SymbolTable
}

impl Emitter {
    pub fn new(symbol_table: SymbolTable) -> Self {
        Self {
            errors: Vec::new(),
            symbol_table
        }
    }

    pub fn emit(&mut self, ast: Vec<Node>) -> Vec<Chunk> {
        let mut chunks: Vec<Chunk> = Vec::new();
        for instructions in ast {
            if let Node::Pipeline(stations) = instructions {
                let mut chunk = Chunk { instructions: Vec::new(), constants: Vec::new() };
                self.create_chunk(stations, &mut chunk);
                chunks.push(chunk)
            }
        }
        chunks
    }

    fn create_chunk(&mut self, stations: Vec<Node>, chunk: &mut Chunk) {
        for station in stations {
            match station {
                Node::Literal(value) => {
                    chunk.constants.push(value);
                    chunk.instructions.push(Instruction::Load(chunk.constants.len() as u16 - 1))
                },
                Node::Apply {operator, arguments} => {
                    if let Node::Symbol(name) = *operator {
                        let result = self.symbol_table.resolve(&name);
                        match result {
                            Ok(SymbolType::Builtin(index)) => chunk.instructions.push(Instruction::BuiltinCall(index)),
                            Ok(SymbolType::Local(scope, index)) => chunk.instructions.push(Instruction::Call(scope, index)),
                            Ok(SymbolType::Global(index)) => chunk.instructions.push(Instruction::GlobalCall(index)),
                            Err(error) => self.errors.push(error)
                        }
                    }
                    self.create_chunk(arguments, chunk);

                },
                _ => todo!()
            }
        }
    }
}
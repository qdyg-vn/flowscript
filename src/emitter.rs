use crate::error_handler::Error;
use crate::instructions::{Chunk, Instruction};
use crate::node::Node;
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
        let mut map: Vec<Chunk> = Vec::new();
        self.symbol_table.new_scope();
        for instructions in ast {
            if let Node::Pipeline(stations) = instructions {
                let mut chunk = Chunk { instructions: Vec::new(), constants: Vec::new(), arity: 0 };
                self.create_chunk(stations, &mut chunk);
                map.push(chunk)
            }
        }
        map
    }

    fn create_chunk(&mut self, stations: Vec<Node>, chunk: &mut Chunk) {
        for station in stations {
            match station {
                Node::Literal(value) => {
                    chunk.constants.push(value);
                    chunk.instructions.push(Instruction::Load(chunk.constants.len() as u16 - 1))
                },
                Node::Apply {operator, arguments} => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    if let Node::Symbol(name) = *operator {
                        let result = self.symbol_table.resolve(&name);
                        match result {
                            Ok(SymbolType::Builtin(index)) => chunk.instructions.push(Instruction::BuiltinCall(index, arity)),
                            Ok(SymbolType::Scope(scope, index)) => chunk.instructions.push(Instruction::Call(scope, index, arity)),
                            Err(error) => self.errors.push(error)
                        }
                    }
                },
                Node::RelativeReference(x, y) => chunk.instructions.push(Instruction::RelativeReference(x, y)),
                Node::Variable(name) => {
                    match self.symbol_table.add_variable(name, 0) {
                        Ok(SymbolType::Scope(scope, index)) => {
                            chunk.instructions.push(Instruction::Store(scope, index));
                            chunk.arity += 1
                        },
                        Err(SymbolType::Scope(scope, index)) => chunk.instructions.push(Instruction::Store(scope, index)),
                        _ => unreachable!()
                    }
                },
                _ => todo!()
            }
        }
    }
}
use crate::error_handler::Error;
use crate::instructions::{Chunk, Instruction};
use crate::node::Node;
use crate::symbol_table::{SymbolTable, SymbolType};
use crate::value::Value;

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
                Node::Assignment(name) => {
                    match self.symbol_table.add_variable(name, 0) {
                        Ok(SymbolType::Scope(scope, index)) => {
                            chunk.instructions.push(Instruction::Store(scope, index));
                            chunk.arity += 1
                        },
                        Err(SymbolType::Scope(scope, index)) => chunk.instructions.push(Instruction::Store(scope, index)),
                        _ => unreachable!()
                    }
                },
                Node::Variable(name) => {
                    match self.symbol_table.resolve(&name) {
                        Ok(SymbolType::Scope(scope, index)) => chunk.instructions.push(Instruction::LoadVariable(scope, index)),
                        Err(error) => todo!(),
                        _ => unreachable!()
                    }
                },
                Node::DefineFunction {operator, arguments, body} => {
                    match self.symbol_table.add_variable(operator, self.symbol_table.scopes.len() as u16 - 1) {
                        Ok(SymbolType::Scope(scope, index)) => {
                            chunk.instructions.push(Instruction::DefineFunction(scope, index));
                            chunk.arity += 1
                        },
                        Err(SymbolType::Scope(scope, index)) => chunk.instructions.push(Instruction::DefineFunction(scope, index)),
                        _ => unreachable!()
                    }
                    self.symbol_table.new_scope();
                    let mut child_chunk = Chunk { instructions: Vec::new(), constants: Vec::new(), arity: 0 };
                    self.create_chunk(arguments, &mut child_chunk);
                    self.create_chunk(body, &mut child_chunk);
                    self.symbol_table.scopes.pop();
                    chunk.constants.push(Value::Function(Box::from(child_chunk)))
                },
                Node::Pipeline(stations) => self.create_chunk(stations, chunk),
                Node::Condition {condition, if_body, else_body} => {
                    self.create_chunk(condition, chunk);
                    let mut if_chunk = Chunk { instructions: Vec::new(), constants: Vec::new(), arity: 0 };
                    self.symbol_table.new_scope();
                    self.create_chunk(if_body, &mut if_chunk);
                    self.symbol_table.scopes.pop();
                    chunk.constants.push(Value::Function(Box::from(if_chunk)));
                    let if_index = chunk.constants.len() as u16 - 1;
                    let mut else_chunk = Chunk { instructions: Vec::new(), constants: Vec::new(), arity: 0 };
                    self.symbol_table.new_scope();
                    self.create_chunk(else_body, &mut else_chunk);
                    self.symbol_table.scopes.pop();
                    chunk.constants.push(Value::Function(Box::from(else_chunk)));
                    let else_index = chunk.constants.len() as u16 - 1;
                    chunk.instructions.push(Instruction::Condition(if_index, else_index))
                }
                _ => todo!(),
            }
        }
    }
}
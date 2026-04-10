use crate::constants_pool::ConstantsPool;
use crate::error_handler::Error;
use crate::instructions::{Chunk, Instruction};
use crate::node::Node;
use crate::symbol_table::{SymbolTable, SymbolType};
use crate::value::Value;
use std::rc::Rc;

pub struct Emitter {
    errors: Vec<Error>,
    symbol_table: SymbolTable,
    constants_pool: ConstantsPool,
}

impl Emitter {
    pub fn new(symbol_table: SymbolTable, constants_pool: ConstantsPool) -> Self {
        Self {
            errors: Vec::new(),
            symbol_table,
            constants_pool,
        }
    }

    pub fn emit(mut self, ast: Vec<Node>) -> (ConstantsPool, Vec<Chunk>) {
        let mut map: Vec<Chunk> = Vec::new();
        self.symbol_table.new_scope();
        for instructions in ast {
            if let Node::Pipeline(stations) = instructions {
                let mut chunk = Chunk { instructions: Vec::new(), arity: 0 };
                self.create_chunk(stations, &mut chunk);
                map.push(chunk)
            }
        }
        (self.constants_pool, map)
    }

    fn create_chunk(&mut self, stations: Vec<Node>, chunk: &mut Chunk) {
        for station in stations {
            match station {
                Node::Literal(value) => {
                    let index = self.constants_pool.add_constant(value);
                    chunk.instructions.push(Instruction::Load(index as u16))
                },
                Node::Apply {operator, arguments} => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    if let Node::Symbol(name) = *operator {
                        let result = self.symbol_table.resolve(&name);
                        match result {
                            Ok(SymbolType::Builtin(index)) => chunk.instructions.push(Instruction::BuiltinCall(index, arity)),
                            Ok(SymbolType::Scope(scope, index)) => chunk.instructions.push(Instruction::Call(scope, index)),
                            Err(error) => self.errors.push(error)
                        }
                    }
                },
                Node::RelativeReference(x, y) => chunk.instructions.push(Instruction::RelativeReference(x, y)),
                Node::Assignment(name) => {
                    match self.symbol_table.add_variable(name) {
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
                    let index = match self.symbol_table.add_variable(operator) {
                        Ok(SymbolType::Scope(scope, index)) => {
                            chunk.arity += 1;
                            index
                        },
                        Err(SymbolType::Scope(scope, index)) => {
                            index
                        },
                        _ => unreachable!()
                    };
                    self.symbol_table.new_scope();
                    let mut child_chunk = Chunk { instructions: Vec::new(), arity: 0 };
                    self.create_chunk(arguments, &mut child_chunk);
                    self.create_chunk(body, &mut child_chunk);
                    self.symbol_table.scopes.pop();
                    let body_index = self.constants_pool.add_constant(Value::Function(Rc::from(child_chunk)));
                    chunk.instructions.push(Instruction::DefineFunction(index, body_index as u16));
                },
                Node::Pipeline(stations) => self.create_chunk(stations, chunk),
                Node::Condition {condition, if_body, else_body} => self.emit_condition(condition, if_body, else_body, chunk),
                Node::Return(value) => {
                    self.create_chunk(vec![*value], chunk);
                    chunk.instructions.push(Instruction::Return)
                },
                _ => todo!(),
            }
        }
    }

    fn emit_condition(&mut self, condition: Vec<Node>, if_body: Vec<Node>, else_body: Vec<Node>, chunk: &mut Chunk) {
        self.create_chunk(condition, chunk);
        let if_position = chunk.instructions.len();
        chunk.instructions.push(Instruction::JumpIfFalse(0));
        self.create_chunk(if_body, chunk);
        chunk.instructions.push(Instruction::Jump(0));
        let else_position = chunk.instructions.len();
        self.create_chunk(else_body, chunk);
        let end_position = chunk.instructions.len();
        if let Instruction::JumpIfFalse(target) = &mut chunk.instructions[if_position] { *target = else_position as u16 };
        if let Instruction::Jump(target) = &mut chunk.instructions[else_position - 1] { *target = end_position as u16 };
    }
}
use crate::constants_pool::ConstantsPool;
use crate::error_handler::{Error, ErrorHandler, SemanticError, SemanticErrorType};
use crate::instructions::{Chunk, Instruction};
use crate::node::Node;
use crate::symbol_table::{SymbolTable, SymbolType};
use crate::value::HeavyValue;

pub struct Emitter {
    error_handler: ErrorHandler,
    symbol_table: SymbolTable,
    constants_pool: ConstantsPool,
}

impl Emitter {
    pub fn new(error_handler: ErrorHandler, symbol_table: SymbolTable, constants_pool: ConstantsPool) -> Self {
        Self {
            error_handler,
            symbol_table,
            constants_pool,
        }
    }

    pub fn emit(mut self, ast: Vec<Node>) -> (ErrorHandler, ConstantsPool, Vec<Chunk>, usize) {
        let mut total_arity = 0;
        let mut map: Vec<Chunk> = Vec::new();
        for instructions in ast {
            if let Node::Pipeline(stations) = instructions {
                let mut chunk = Chunk { instructions: Vec::new(), arity: 0 };
                self.create_chunk(stations, &mut chunk);
                total_arity += chunk.arity;
                map.push(chunk)
            }
        }
        if !self.error_handler.errors.is_empty() { self.error_handler.report_exit() }
        (self.error_handler, self.constants_pool, map, total_arity as usize)
    }

    fn create_chunk(&mut self, stations: Vec<Node>, chunk: &mut Chunk) {
        for station in stations {
            match station {
                Node::Literal(value) => {
                    let index = self.constants_pool.add_constant(value);
                    chunk.instructions.push(Instruction::Load(index as u32))
                },
                Node::HeavyLiteral(heavy_value) => {
                    let index = self.constants_pool.add_heavy_constant(heavy_value);
                    chunk.instructions.push(Instruction::Load(index as u32))
                }
                Node::Apply {operator, arguments} => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    if let Node::Symbol(name) = *operator {
                        let result = self.symbol_table.resolve(&name);
                        match result {
                            Ok(SymbolType::Builtin(index)) => chunk.instructions.push(Instruction::BuiltinCall(index, arity)),
                            Ok(SymbolType::Scope(scope, index)) => chunk.instructions.push(Instruction::Call(scope, index)),
                            Err(error) => self.error_handler.errors.push(Error::SemanticError(error))
                        }
                    }
                },
                Node::RelativeReference(x, y) => chunk.instructions.push(Instruction::RelativeReference(x, y)),
                Node::Assignment(name) => {
                    match self.symbol_table.add_variable(name) {
                        Ok(SymbolType::Scope(_, index)) => {
                            chunk.instructions.push(Instruction::Store(index));
                            chunk.arity += 1
                        },
                        Err(SymbolType::Scope(_, index)) => chunk.instructions.push(Instruction::Store(index)),
                        _ => unreachable!()
                    }
                },
                Node::Variable(name) => {
                    match self.symbol_table.resolve(&name) {
                        Ok(SymbolType::Scope(_, index)) => chunk.instructions.push(Instruction::LoadVariable(index)),
                        Err(error) => self.error_handler.errors.push(Error::SemanticError(error)),
                        _ => unreachable!()
                    }
                },
                Node::DefineFunction {operator, arguments, body} => {
                    let index = match self.symbol_table.add_variable(operator) {
                        Ok(SymbolType::Scope(_, index)) => {
                            chunk.arity += 1;
                            index
                        },
                        Err(SymbolType::Scope(_, index)) => {
                            index
                        },
                        _ => unreachable!()
                    };
                    self.symbol_table.new_scope();
                    let mut child_chunk = Chunk { instructions: Vec::new(), arity: 0 };
                    for argument in arguments {
                        let Node::Assignment(name) = argument else { unreachable!() };
                        match self.symbol_table.add_variable(name.clone()) {
                            Ok(_) => child_chunk.arity += 1,
                            Err(SymbolType::Scope(_, _)) => self.error_handler.errors.push(Error::SemanticError(SemanticError {kind: SemanticErrorType::DuplicateParameter(name)})),
                            _ => unreachable!()
                        }
                    }
                    self.create_chunk(body, &mut child_chunk);
                    self.symbol_table.scopes.pop();
                    let body_index = self.constants_pool.add_heavy_constant(HeavyValue::Function(child_chunk));
                    chunk.instructions.push(Instruction::DefineFunction(index, body_index as u16));
                },
                Node::Pipeline(stations) => self.create_chunk(stations, chunk),
                Node::Condition {branches, final_branch} => self.emit_condition(branches, final_branch, chunk),
                Node::Return(value) => {
                    self.create_chunk(vec![*value], chunk);
                    chunk.instructions.push(Instruction::Return)
                },
                Node::Array(elements) => {
                    let count = elements.len() as u32;
                    self.create_chunk(elements, chunk);
                    chunk.instructions.push(Instruction::Array(count))
                }
                _ => unreachable!(),
            }
        }
    }

    fn emit_condition(&mut self, branches: Vec<(Vec<Node>, Vec<Node>)>, final_branch: Vec<Node>, chunk: &mut Chunk) {
        let mut complete_positions = Vec::with_capacity(branches.len());
        for (condition, body) in branches {
            self.create_chunk(condition, chunk);
            let branch_position = chunk.instructions.len();
            chunk.instructions.push(Instruction::JumpIfFalse(0));
            self.create_chunk(body, chunk);
            complete_positions.push(chunk.instructions.len());
            chunk.instructions.push(Instruction::Jump(0));
            let next_branch = chunk.instructions.len();
            if let Instruction::JumpIfFalse(target) = &mut chunk.instructions[branch_position] { *target = next_branch as u16 };
        }
        self.create_chunk(final_branch, chunk);
        let end_position = chunk.instructions.len();
        for index in complete_positions {
            if let Instruction::Jump(target) = &mut chunk.instructions[index] { *target = end_position as u16 };
        }
    }
}
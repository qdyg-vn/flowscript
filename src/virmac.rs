use crate::builtins::{get_builtin, BuiltinFunction};
use crate::instructions::{Chunk, Instruction};
use crate::value::Value;

pub struct VirMac {
    stack_position: usize,
    stations_output: Vec<Value>,
    variables: Vec<Value>,
    constants_pool: Vec<Value>,
}

impl VirMac {
    pub fn new(constants_pool: Vec<Value>) -> Self {
        Self {
            stack_position: 0,
            stations_output: Vec::new(),
            variables: Vec::new(),
            constants_pool
        }
    }

    pub fn execute(&mut self, map: Vec<Chunk>) -> Vec<Value> {
        let mut stack = vec![Value::Nil; 1024];
        for chunk in map {
            self.variables.resize(chunk.arity as usize, Value::Nil);
            let chunk_size = (&chunk.instructions).len();
            let mut instruction_position = 0;
            while instruction_position < chunk_size {
                self.dispatch_instruction(chunk.instructions[instruction_position], &mut instruction_position, &mut stack, 0);
                instruction_position += 1;
            }
        }
        stack
    }

    fn dispatch_instruction(&mut self, instruction: Instruction, instruction_position: &mut usize, stack: &mut Vec<Value>, base_pointer: usize) {
        if stack.len() <= self.stack_position { stack.resize(stack.len() * 2, Value::Nil) }
        match instruction {
            Instruction::Load(index) => {
                if stack.len() <= self.stack_position { stack.resize(stack.len() * 2, Value::Nil) }
                stack[self.stack_position] = self.constants_pool[index as usize].clone();
                self.stations_output.push(self.constants_pool[index as usize].clone());
                self.stack_position += 1;
            },
            Instruction::BuiltinCall(index, arity) => {
                let start = self.stack_position - arity as usize;
                let end = self.stack_position;
                self.execute_builtin_function(index, stack, start, end);
            },
            Instruction::RelativeReference(x, y) => {
                if self.stations_output.len() < x as usize {
                    todo!("There is no station at position {}", x)
                }
                if stack.len() <= self.stack_position { stack.resize(stack.len() * 2, Value::Nil) }
                let output = self.stations_output[self.stations_output.len() - x as usize].clone();
                if y != 0 {
                    todo!("Currently under development")
                } else {
                    stack[self.stack_position] = output;
                }
                self.stack_position += 1
            },
            Instruction::Store(scope, index) => { self.variables[base_pointer + index as usize] = stack[self.stack_position - 1].clone(); println!("64 {:?}", self.variables) },
            Instruction::LoadVariable(scope, index) => {
                stack[self.stack_position] = self.variables[base_pointer + index as usize].clone();
                self.stack_position += 1
            },
            Instruction::DefineFunction(scope, index, body_index) => {
                self.variables[base_pointer + index as usize] = self.constants_pool[body_index as usize].clone();
            },
            Instruction::Call(scope, index, arity) => {
                let start = self.stack_position - arity as usize;
                let end = self.stack_position;
                let chunk = match self.variables[base_pointer + index as usize].clone() {
                    Value::Function(chunk_box) => *chunk_box,
                    _ => todo!()
                };
                let mut child_instruction_position = 0;
                let base_pointer = self.variables.len();
                for instruction in chunk.instructions {
                    self.variables.resize(base_pointer + chunk.arity as usize, Value::Nil);
                    self.dispatch_instruction(instruction, &mut child_instruction_position, stack, base_pointer);
                    child_instruction_position += 1
                }
            },
            Instruction::JumpIfFalse(position) => if stack[self.stack_position - 1] == Value::Boolean(false) { *instruction_position = position as usize },
            Instruction::Jump(position) => *instruction_position = position as usize - 1,
        }
    }

    fn execute_builtin_function(&mut self, index: u16, stack: &mut Vec<Value>, start: usize, end: usize) {
        if let Some(arguments) = stack.get(start..end) {
            let builtin = get_builtin(index);
            match builtin.function {
                BuiltinFunction::Math(function) => {
                    match function(arguments) {
                        Ok(value) => {
                            self.stations_output.push(value.clone());
                            stack[start] = value;
                            self.stack_position = start + 1;
                        },
                        Err(error) => todo!()
                    };
                },
                BuiltinFunction::IO(function) => function(arguments),
                BuiltinFunction::Casting(function) | BuiltinFunction::Compare(function) => {
                    let value = function(arguments);
                    self.stations_output.push(value.clone());
                    stack[start] = value;
                    self.stack_position = start + 1
                },
            }
        } else {
            todo!("Stack out of bounds: start {} end {}", start, end)
        }
    }
}
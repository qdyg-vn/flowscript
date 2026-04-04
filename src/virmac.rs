use crate::builtins::{get_builtin, BuiltinFunction};
use crate::instructions::{Chunk, Instruction};
use crate::value::Value;

#[derive(Default)]
pub struct VirMac {
    position: usize,
    stations_output: Vec<Value>,
    variables: Vec<Value>
}

impl VirMac {
    pub fn execute(&mut self, map: Vec<Chunk>) -> Vec<Value> {
        let mut stack = vec![Value::Nil; 1024];
        for chunk in map {
            self.variables.resize(chunk.arity as usize, Value::Nil);
            for instruction in chunk.instructions {
                self.dispatch_instruction(instruction, &chunk.constants, &mut stack)
            }
        }
        stack
    }

    fn dispatch_instruction(&mut self, instruction: Instruction, constants: &Vec<Value>, stack: &mut Vec<Value>) {
        if stack.len() <= self.position { stack.resize(stack.len() * 2, Value::Nil) }
        match instruction {
            Instruction::Load(index) => {
                if stack.len() <= self.position { stack.resize(stack.len() * 2, Value::Nil) }
                stack[self.position] = constants[index as usize].clone();
                self.stations_output.push(constants[index as usize].clone());
                self.position += 1
            },
            Instruction::BuiltinCall(index, arity) => {
                let start = self.position - arity as usize;
                let end = self.position;
                self.execute_builtin_function(index, stack, start, end);
            },
            Instruction::RelativeReference(x, y) => {
                if self.stations_output.len() < x as usize {
                    todo!("There is no station at position {}", x)
                }
                if stack.len() <= self.position { stack.resize(stack.len() * 2, Value::Nil) }
                let output = self.stations_output[self.stations_output.len() - x as usize].clone();
                if y != 0 {
                    todo!("Currently under development")
                } else {
                    stack[self.position] = output;
                }
                self.position += 1
            },
            Instruction::Store(scope, index) => self.variables[index as usize] = stack[self.position - 1].clone(),
            Instruction::LoadVariable(scope, index) => {
                stack[self.position] = self.variables[index as usize].clone();
                self.position += 1
            },
            Instruction::DefineFunction(scope, index) => {
                self.variables[index as usize] = constants[index as usize].clone()
            }
            _ => todo!()
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
                            self.position = start + 1;
                        },
                        Err(error) => todo!()
                    };
                },
                BuiltinFunction::IO(function) => function(arguments),
                BuiltinFunction::Casting(function) => {
                    let value = function(arguments);
                    self.stations_output.push(value.clone());
                    stack[start] = value;
                    self.position = start + 1
                },
            }
        } else {
            todo!("Stack out of bounds: start {} end {}", start, end)
        }
    }
}
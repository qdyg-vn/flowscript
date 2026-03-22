use crate::builtins::{get_builtin, BuiltinFunction};
use crate::instructions::{Chunk, Instruction};
use crate::value::Value;

#[derive(Default)]
pub struct VirMac {
    postion: usize,
    stations_output: Vec<Value>,
}

impl VirMac {
    pub fn execute(&mut self, map: Vec<Chunk>) -> Vec<Value> {
        let mut stack = vec![Value::Nil; 1024];
        for chunk in map {
            for instruction in chunk.instructions {
                match instruction {
                    Instruction::Load(index) => {
                        if stack.len() <= self.postion { stack.resize(stack.len() * 2, Value::Nil) }
                        stack[self.postion] = chunk.constants[index as usize].clone();
                        self.postion += 1
                    },
                    Instruction::BuiltinCall(index, arity) => {
                        let start = self.postion - arity as usize;
                        let end = self.postion - 1;
                        self.execute_builtin_function(index, &mut stack, start, end);
                    },
                    Instruction::RelativeReference => {
                        if stack.len() <= self.postion { stack.resize(stack.len() * 2, Value::Nil) }
                        stack[self.postion] = self.stations_output[self.stations_output.len() - 1].clone();
                        self.postion += 1
                    },
                    _ => todo!()
                }
            }
        }
        stack
    }

    fn execute_builtin_function(&mut self, index: u16, stack: &mut Vec<Value>, start: usize, end: usize) {
        if let Some(arguments) = stack.get(start..=end) {
            let builtin = get_builtin(index);
            match builtin.function {
                BuiltinFunction::Math(function) => {
                    match function(arguments) {
                        Ok(value) => {
                            self.stations_output.push(value.clone());
                            stack[start] = value;
                            self.postion = start
                        },
                        Err(error) => todo!()
                    };
                },
                BuiltinFunction::IO(function) => function(arguments)
            }
        } else {
            todo!("Stack out of bounds: start {} end {}", start, end)
        }
    }
}
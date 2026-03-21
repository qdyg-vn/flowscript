use crate::builtins::{get_builtin, BuiltinFunction};
use crate::instructions::{Chunk, Instruction};
use crate::value::Value;

pub struct VirMac {

}

impl VirMac {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&mut self, map: Vec<Chunk>) -> Vec<Value> {
        let mut stack = Vec::new();
        for chunk in map {
            for instruction in chunk.instructions {
                match instruction {
                    Instruction::Load(index) => stack.push(chunk.constants[index as usize].clone()),
                    Instruction::BuiltinCall(index, arity) => {
                        let start = stack.len() - arity as usize;
                        let end = stack.len() - 1;
                        self.execute_builtin_function(index, &mut stack, start, end);
                    }
                    _ => todo!()
                }
            }
        }
        stack
    }

    fn execute_builtin_function(&self, index: u16, stack: &mut Vec<Value>, start: usize, end: usize) {
        if let Some(arguments) = stack.get(start..=end) {
            let builtin = get_builtin(index);
            match builtin.function {
                BuiltinFunction::Math(function) => {
                    match function(arguments) {
                        Ok(value) => stack.push(value),
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
use crate::builtins::{get_builtin, BuiltinFunction};
use crate::instructions::{Chunk, Instruction};
use crate::value::Value;

pub struct VirMac {
    stations_output: Vec<Value>,
    base_pointers: Vec<usize>,
    constants_pool: Vec<Value>,
}

impl VirMac {
    pub fn new(constants_pool: Vec<Value>) -> Self {
        Self {
            stations_output: Vec::new(),
            base_pointers: Vec::new(),
            constants_pool
        }
    }

    pub fn execute(&mut self, map: Vec<Chunk>) -> Vec<Value> {
        let mut stack = vec![Value::Nil; 1024];
        for chunk in map {
            let chunk_size = (&chunk.instructions).len();
            let mut stack_position = chunk.arity as usize;
            let mut instruction_position = 0;
            self.base_pointers.push(0);
            while instruction_position < chunk_size {
                self.dispatch_instruction(chunk.instructions[instruction_position], &mut instruction_position, &mut stack, &mut stack_position, 0);
                instruction_position += 1;
            }
        }
        stack
    }

    fn dispatch_instruction(&mut self, instruction: Instruction, instruction_position: &mut usize, stack: &mut Vec<Value>, stack_position: &mut usize, base_pointer: usize) {
        if stack.len() <= *stack_position { stack.resize(stack.len() * 2, Value::Nil) }
        match instruction {
            Instruction::Load(index) => {
                stack[*stack_position] = self.constants_pool[index as usize].clone();
                self.stations_output.push(self.constants_pool[index as usize].clone());
                *stack_position += 1;
            },
            Instruction::BuiltinCall(index, arity) => {
                let start = *stack_position - arity as usize;
                let end = *stack_position;
                self.execute_builtin_function(index, stack, stack_position, start, end);
            },
            Instruction::RelativeReference(x, y) => {
                if self.stations_output.len() < x as usize {
                    todo!("There is no station at position {}", x)
                }
                let output = self.stations_output[self.stations_output.len() - x as usize].clone();
                if y != 0 {
                    todo!("Currently under development")
                } else {
                    stack[*stack_position] = output;
                }
                *stack_position += 1
            },
            Instruction::Store(scope, index) => stack[base_pointer + index as usize] = stack[*stack_position - 1].clone(),
            Instruction::LoadVariable(scope, index) => {
                stack[*stack_position] = stack[base_pointer + index as usize].clone();
                *stack_position += 1;
            },
            Instruction::DefineFunction(index, body_index) => stack[base_pointer + index as usize] = self.constants_pool[body_index as usize].clone(),
            Instruction::Call(scope, index) => {
                let child_chunk = match stack[self.base_pointers[scope as usize] + index as usize].clone() {
                    Value::Function(chunk_box) => *chunk_box,
                    _ => todo!()
                };
                let mut child_instruction_position = 0;
                let mut child_stack_position = *stack_position;
                let child_base_pointer = child_stack_position - child_chunk.arity as usize;
                let child_chunk_size = (&child_chunk.instructions).len();
                self.base_pointers.push(child_base_pointer);
                while child_instruction_position < child_chunk_size {
                    let child_instruction = child_chunk.instructions[child_instruction_position];
                    self.dispatch_instruction(child_instruction, &mut child_instruction_position, stack, &mut child_stack_position, child_base_pointer);
                    if matches!(child_instruction, Instruction::Return) {
                        self.stations_output.push(stack[child_base_pointer].clone());
                        break
                    }
                    child_instruction_position += 1;
                };
            },
            Instruction::JumpIfFalse(position) => {
                if stack[*stack_position - 1] == Value::Boolean(false) {
                    *instruction_position = position as usize - 1
                };
                *stack_position -= 1
            },
            Instruction::Jump(position) => *instruction_position = position as usize - 1,
            Instruction::Return => stack[base_pointer] = stack[*stack_position - 1].clone(),
        }
    }

    fn execute_builtin_function(&mut self, index: u16, stack: &mut Vec<Value>, stack_position: &mut usize, start: usize, end: usize) {
        if let Some(arguments) = stack.get(start..end) {
            let builtin = get_builtin(index);
            match builtin.function {
                BuiltinFunction::Math(function) => {
                    match function(arguments) {
                        Ok(value) => {
                            self.stations_output.push(value.clone());
                            stack[start] = value;
                            *stack_position = start + 1;
                        },
                        Err(error) => todo!()
                    };
                },
                BuiltinFunction::IO(function) => function(arguments),
                BuiltinFunction::Casting(function) | BuiltinFunction::Compare(function) => {
                    let value = function(arguments);
                    self.stations_output.push(value.clone());
                    stack[start] = value;
                    *stack_position = start + 1;
                },
            }
        } else {
            todo!("Stack out of bounds: start {} end {}", start, end)
        }
    }
}
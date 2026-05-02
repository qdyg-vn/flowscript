use crate::builtins::{get_builtin, BuiltinFunction};
use crate::instructions::Bytecode;
use crate::value::Value;
use crate::error_handler::{ErrorHandler, Error, RuntimeError, RuntimeErrorType};
use crate::memory::Memory;

pub struct VirMac {
    stations_output: Vec<Value>,
    base_pointers: Vec<usize>,
    memory: Memory,
    constants_pool: Vec<Value>,
    error_handler: ErrorHandler,
}

impl VirMac {
    pub fn new(memory: Memory, error_handler: ErrorHandler, constants_pool: Vec<Value>) -> Self {
        Self {
            stations_output: Vec::new(),
            base_pointers: vec![0],
            memory,
            constants_pool,
            error_handler,
        }
    }

    pub fn execute(&mut self, map: Vec<Vec<u8>>, total_arity: usize) -> Vec<Value> {
        let mut stack = vec![Value::Nil; 1024];
        for mut chunk in map {
            self.stations_output.clear();
            let chunk_size = chunk.len();
            let mut stack_position = total_arity;
            let mut instruction_position = 0;
            while instruction_position < chunk_size {
                self.dispatch_instruction(&mut chunk, &mut instruction_position, &mut stack, &mut stack_position, 0);
                instruction_position += 1;
            }
        }
        stack
    }

    fn dispatch_instruction(&mut self, chunk: &mut Vec<u8>, instruction_position: &mut usize, stack: &mut Vec<Value>, stack_position: &mut usize, base_pointer: usize) {
        if stack.len() <= *stack_position { stack.resize(stack.len() * 2, Value::Nil) }
        match chunk[*instruction_position] {
            Bytecode::LOAD => {
                let index = u32::from_le_bytes(chunk[*instruction_position + 1..=*instruction_position + 4].try_into().unwrap());
                stack[*stack_position] = self.constants_pool[index as usize].clone();
                self.stations_output.push(self.constants_pool[index as usize].clone());
                *stack_position += 1;
                *instruction_position += 3
            },
            Bytecode::LOADVARIABLE => {
                let index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                stack[*stack_position] = stack[base_pointer + index as usize].clone();
                *stack_position += 1;
                *instruction_position += 3
            },
            Bytecode::JUMP => {
                let position = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                *instruction_position = position as usize
            },
            Bytecode::JUMPIFFALSE => {
                let position = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                if stack[*stack_position - 1] == Value::Boolean(false) {
                    *instruction_position = position as usize - 1
                };
                *stack_position -= 1
            },
            Bytecode::STORE => {
                let index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                stack.swap(base_pointer + index as usize, *stack_position - 1);
                *instruction_position += 3
            },
            Bytecode::BUILTINCALL => {
                let index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                let arity = u16::from_le_bytes([chunk[*instruction_position + 3], chunk[*instruction_position + 4]]);
                let start = *stack_position - arity as usize;
                let end = *stack_position;
                self.execute_builtin_function(index, stack, stack_position, start, end);
                *instruction_position += 5
            },
            Bytecode::CALL => {
                let scope = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                let index = u16::from_le_bytes([chunk[*instruction_position + 3], chunk[*instruction_position + 4]]);
                let function_index = match &stack[self.base_pointers[scope as usize] + index as usize] {
                    Value::FunctionPointer(index) => *index,
                    _ => todo!()
                };
                let (mut function, arity, length) = self.to_function(function_index as usize);
                let mut function_instruction_position = 0;
                let mut child_stack_position = *stack_position;
                let child_base_pointer = child_stack_position - arity as usize;
                self.base_pointers.push(child_base_pointer);
                while function_instruction_position < length {
                    if matches!(function[function_instruction_position], Bytecode::RETURN) {
                        self.dispatch_instruction(&mut function, &mut function_instruction_position, stack, &mut child_stack_position, child_base_pointer);
                        self.stations_output.push(stack[child_base_pointer].clone());
                        break
                    }
                    self.dispatch_instruction(&mut function, &mut function_instruction_position, stack, &mut child_stack_position, child_base_pointer);
                    function_instruction_position += 1;
                };
            },
            Bytecode::RELATIVEREFERENCE => {
                let x = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                let y = u16::from_le_bytes([chunk[*instruction_position + 3], chunk[*instruction_position + 4]]);
                if self.stations_output.len() < x as usize {
                    self.error_handler.fatal(Error::RuntimeError(RuntimeError {kind: RuntimeErrorType::MissingStation(x)}))
                }
                let output = self.stations_output[self.stations_output.len() - x as usize].clone();
                if y != 0 {
                    todo!("Currently under development")
                } else {
                    stack[*stack_position] = output;
                }
                *stack_position += 1;
                *instruction_position += 5;
            },
            Bytecode::RETURN => {
                stack.swap(base_pointer, *stack_position - 1);
                *instruction_position += 1
            },
            Bytecode::DEFINEFUNCTION => {
                let index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                let body_index = u16::from_le_bytes([chunk[*instruction_position + 3], chunk[*instruction_position + 4]]);
                stack[base_pointer + index as usize] = self.constants_pool[body_index as usize].clone();
                *instruction_position += 5
            },
            Bytecode::ARRAY => {
                let count = u32::from_le_bytes(chunk[*instruction_position + 1..=*instruction_position + 4].try_into().unwrap());
                let start = *stack_position - count as usize;
                let mut array = Vec::new();
                for value in stack[start..*stack_position].iter() {
                    array.extend_from_slice(&value.to_byte())
                }
                let index = self.memory.permanent_space.len();
                self.memory.permanent_space.extend_from_slice(&array.len().to_le_bytes());
                self.memory.permanent_space.extend_from_slice(&array);
                stack[start] = Value::ArrayPointer(index as u32);
                *stack_position = start + 1;
                self.stations_output.push(stack[*stack_position - 1].clone());
                *instruction_position += 5
            },
            _ => unreachable!()
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
                BuiltinFunction::Introspection(function) => {
                    match function(arguments) {
                        Ok(value) => {
                            self.stations_output.push(value.clone());
                            stack[start] = value;
                            *stack_position = start + 1;
                        },
                        Err(error) => self.error_handler.fatal(error)
                    };
                }
            }
        } else {
            self.error_handler.fatal(Error::RuntimeError(RuntimeError {kind: RuntimeErrorType::OutOfBounds(start, end)}))
        }
    }

    fn to_function(&self, start: usize) -> (Vec<u8>, u16, usize) {
        let length = u64::from_le_bytes(self.memory.permanent_space[start..start + 8].try_into().unwrap());
        let arity = u16::from_le_bytes([self.memory.permanent_space[start + 8], self.memory.permanent_space[start + 9]]);
        let function = self.memory.permanent_space[start + 10..start + length as usize].to_vec();
        (function, arity, length as usize)
    }
}
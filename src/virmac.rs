use crate::builtins::{get_builtin, BuiltinFunction};
use crate::instructions::Bytecode;
use crate::value::{LightValue, Value};
use crate::error_handler::{ErrorHandler, Error, RuntimeError, RuntimeErrorType};
use crate::memory::Memory;

pub struct VirMac {
    stations_output: Vec<LightValue>,
    base_pointers: Vec<usize>,
    memory: Memory,
    constants_pool: Vec<LightValue>,
    starts : Vec<usize>,
    error_handler: ErrorHandler,
}

impl VirMac {
    pub fn new(memory: Memory, error_handler: ErrorHandler, constants_pool: Vec<LightValue>, starts: Vec<usize>) -> Self {
        Self {
            stations_output: Vec::new(),
            base_pointers: vec![0],
            memory,
            constants_pool,
            starts,
            error_handler,
        }
    }

    pub fn execute(&mut self, map: Vec<Vec<u8>>, total_arity: usize) -> Vec<LightValue> {
        let mut stack = vec![LightValue::Nil; 1024];
        for chunk in map {
            self.stations_output.clear();
            let chunk_size = chunk.len();
            let mut stack_position = total_arity;
            let mut instruction_position = 0;
            while instruction_position < chunk_size {
                self.dispatch_instruction(&chunk, &mut instruction_position, &mut stack, &mut stack_position, 0);
            }
        }
        stack
    }

    fn dispatch_instruction(&mut self, chunk: &[u8], instruction_position: &mut usize, stack: &mut Vec<LightValue>, stack_position: &mut usize, base_pointer: usize) {
        if stack.len() <= *stack_position { stack.resize(stack.len() * 2, LightValue::Nil) }
        match chunk[*instruction_position] {
            Bytecode::LOAD => {
                let index = u32::from_le_bytes(chunk[*instruction_position + 1..=*instruction_position + 4].try_into().unwrap());
                stack[*stack_position] = self.constants_pool[index as usize];
                self.stations_output.push(self.constants_pool[index as usize]);
                *stack_position += 1;
                *instruction_position += Bytecode::LOAD_SIZE
            },
            Bytecode::LOAD_VARIABLE => {
                let index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                stack[*stack_position] = stack[base_pointer + index as usize];
                self.stations_output.push(stack[*stack_position]);
                *stack_position += 1;
                *instruction_position += Bytecode::LOAD_VARIABLE_SIZE
            }
            Bytecode::JUMP => {
                let position = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                *instruction_position = position as usize
            },
            Bytecode::JUMP_IF_FALSE => {
                let position = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                if stack[*stack_position - 1] == LightValue::Boolean(false) {
                    *instruction_position = position as usize
                } else {
                    *instruction_position += Bytecode::JUMP_IF_FALSE_SIZE
                };
                *stack_position -= 1
            }
            Bytecode::STORE => {
                let index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                self.stations_output.push(stack[*stack_position - 1]);
                stack[base_pointer + index as usize] = stack[*stack_position - 1];
                *instruction_position += Bytecode::STORE_SIZE
            }
            Bytecode::BUILTIN_CALL => {
                let index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                let arity = u16::from_le_bytes([chunk[*instruction_position + 3], chunk[*instruction_position + 4]]);
                let start = *stack_position - arity as usize;
                let end = *stack_position;
                self.execute_builtin_function(index, stack, stack_position, start, end);
                *instruction_position += Bytecode::BUILTIN_CALL_SIZE
            }
            Bytecode::CALL => {
                let scope = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                let index = u16::from_le_bytes([chunk[*instruction_position + 3], chunk[*instruction_position + 4]]);
                let function_index = match &stack[self.base_pointers[scope as usize] + index as usize] {
                    LightValue::FunctionPointer(index) => *index,
                    _ => todo!()
                };
                let (functions, arities, lengths) = self.to_function(function_index as usize);
                for index in 0..functions.len() {
                    let (function, arity, length) = (&functions[index], arities[index], lengths[index]);
                    let mut function_instruction_position = 0;
                    let mut child_stack_position = *stack_position;
                    let child_base_pointer = child_stack_position - arity as usize;
                    self.base_pointers.push(child_base_pointer);
                    while function_instruction_position < length {
                        if matches!(function[function_instruction_position], Bytecode::RETURN) {
                            self.dispatch_instruction(function, &mut function_instruction_position, stack, &mut child_stack_position, child_base_pointer);
                            self.stations_output.push(stack[child_base_pointer]);
                            break
                        }
                        self.dispatch_instruction(function, &mut function_instruction_position, stack, &mut child_stack_position, child_base_pointer);
                    };
                }
                *instruction_position += Bytecode::CALL_SIZE
            },
            Bytecode::RELATIVE_REFERENCE => {
                let x = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                let y = u16::from_le_bytes([chunk[*instruction_position + 3], chunk[*instruction_position + 4]]);
                if self.stations_output.len() < x as usize {
                    self.error_handler.fatal(Error::RuntimeError(RuntimeError {kind: RuntimeErrorType::MissingStation(x)}))
                }
                let output = self.stations_output[self.stations_output.len() - x as usize];
                if y != 0 {
                    todo!("Currently under development")
                } else {
                    stack[*stack_position] = output;
                }
                *stack_position += 1;
                *instruction_position += Bytecode::RELATIVE_REFERENCE_SIZE;
            }
            Bytecode::RETURN => {
                stack.swap(base_pointer, *stack_position - 1);
                *instruction_position += Bytecode::RETURN_SIZE
            },
            Bytecode::DEFINE_FUNCTION => {
                let index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                let body_index = u16::from_le_bytes([chunk[*instruction_position + 3], chunk[*instruction_position + 4]]);
                stack[base_pointer + index as usize] = self.constants_pool[body_index as usize];
                *instruction_position += Bytecode::DEFINE_FUNCTION_SIZE
            }
            Bytecode::ARRAY => {
                let count = u32::from_le_bytes(chunk[*instruction_position + 1..=*instruction_position + 4].try_into().unwrap());
                let start = *stack_position - count as usize;
                let mut array = Vec::new();
                for value in stack[start..*stack_position].iter() {
                    match value {
                        LightValue::Boolean(boolean) => {
                            array.push(LightValue::BOOLEAN);
                            array.push(*boolean as u8)
                        },
                        LightValue::Nil => {
                            array.push(LightValue::NIL);
                            array.push(0)
                        },
                        LightValue::Float(float) => {
                            array.push(LightValue::FLOAT);
                            array.extend_from_slice(&float.to_le_bytes())
                        },
                        LightValue::Integer(integer) => {
                            array.push(LightValue::INTEGER);
                            array.extend_from_slice(&integer.to_le_bytes())
                        },
                        LightValue::StringPointer(index) => {
                            array.push(LightValue::STRINGPOINTER);
                            array.extend_from_slice(&index.to_le_bytes())
                        },
                        LightValue::ArrayPointer(index) => {
                            array.push(LightValue::ARRAYPOINTER);
                            array.extend_from_slice(&index.to_le_bytes())
                        },
                        _ => unreachable!()
                    }
                }
                let index = self.memory.permanent_space.len();
                self.memory.permanent_space.extend_from_slice(&array.len().to_le_bytes());
                self.memory.permanent_space.extend_from_slice(&array);
                stack[start] = LightValue::ArrayPointer(index as u32);
                *stack_position = start + 1;
                self.stations_output.push(stack[*stack_position - 1]);
                *instruction_position += Bytecode::ARRAY_SIZE
            },
            _ => unreachable!()
        }
    }

    fn execute_builtin_function(&mut self, index: u16, stack: &mut [LightValue], stack_position: &mut usize, start: usize, end: usize) {
        let values = match stack.get(start..end) {
            Some(argument) => argument,
            None => { self.error_handler.fatal(Error::RuntimeError(RuntimeError { kind: RuntimeErrorType::OutOfBounds(start, end) })); unreachable!() }
        };
        let mut arguments = Vec::new();
        for argument in values {
            arguments.push(match argument {
                LightValue::Boolean(boolean) => Value::Boolean(*boolean),
                LightValue::Nil => Value::Nil,
                LightValue::Float(float) => Value::Float(*float),
                LightValue::Integer(integer) => Value::Integer(*integer),
                LightValue::StringPointer(index) => {
                    let string = self.to_string(*index as usize);
                    Value::String(string)
                },
                LightValue::StringHeapPointer(index) => {
                    let string = self.to_heap_string(*index as usize);
                    Value::String(string)
                },
                LightValue::ArrayPointer(index) => Value::Array(self.to_array(*index as usize)),
                LightValue::ArrayHeapPointer(index) => Value::Array(self.to_heap_array(*index as usize)),
                LightValue::FunctionPointer(_) | LightValue::ClosurePointer(_) => unreachable!(),
            })
        }
        let builtin = get_builtin(index);
        match builtin.function {
            BuiltinFunction::Math(function) | BuiltinFunction::Introspection(function) | BuiltinFunction::Compare(function) | BuiltinFunction::Casting(function) => {
                match function(&arguments) {
                    Ok(value) => {
                        let light_value = self.push_into_storage(value, stack);
                        self.stations_output.push(light_value);
                        stack[start] = light_value;
                        *stack_position = start + 1;
                    },
                    Err(error) => self.error_handler.fatal(error)
                };
            },
            BuiltinFunction::IO(function) => function(&arguments),
        }
    }

    fn to_function(&self, index_of_start: usize) -> (Vec<Vec<u8>>, Vec<u16>, Vec<usize>) {
        let mut start = self.starts[index_of_start];
        let length = u64::from_le_bytes(self.memory.permanent_space[start..start + 8].try_into().unwrap());
        let mut functions = Vec::with_capacity(length as usize);
        let mut arities = Vec::with_capacity(length as usize);
        let mut lengths = Vec::with_capacity(length as usize);
        start += 8;
        for _ in 0..length {
            let length = u64::from_le_bytes(self.memory.permanent_space[start..start + 8].try_into().unwrap());
            lengths.push(length as usize);
            let arity = u16::from_le_bytes([self.memory.permanent_space[start + 8], self.memory.permanent_space[start + 9]]);
            arities.push(arity);
            let function = self.memory.permanent_space[start + 10..start + 10 + length as usize].to_vec();
            functions.push(function);
            start += 10 + length as usize
        }
        (functions, arities, lengths)
    }

    fn to_string(&self, index_of_start: usize) -> String {
        let start = self.starts[index_of_start];
        let length = u64::from_le_bytes(self.memory.permanent_space[start..start + 8].try_into().unwrap());
        String::from_utf8(self.memory.permanent_space[start + 8..start + 8 + length as usize].to_vec()).unwrap()
    }

    fn to_heap_string(&self, start: usize) -> String {
        let length = u64::from_le_bytes(self.memory.from_space[start..start + 8].try_into().unwrap());
        String::from_utf8(self.memory.from_space[start + 8..start + 8 + length as usize].to_vec()).unwrap()
    }

    fn to_array(&self, start: usize) -> Vec<Value> {
        self.decode_array(start, &self.memory.permanent_space)
    }

    fn to_heap_array(&self, start: usize) -> Vec<Value> {
        self.decode_array(start, &self.memory.from_space)
    }

    fn decode_array(&self, mut start: usize, space: &[u8]) -> Vec<Value> {
        let length = u64::from_le_bytes(space[start..start + 8].try_into().unwrap());
        let end = start + length as usize;
        start += 8;
        let mut array = Vec::new();
        while start < end {
            array.push(match space[start] {
                LightValue::BOOLEAN => {
                    let boolean = Value::Boolean(space[start + 1] != 0);
                    start += LightValue::BOOLEAN_SIZE;
                    boolean
                },
                LightValue::NIL => {
                    start += LightValue::NIL_SIZE;
                    Value::Nil
                },
                LightValue::FLOAT => {
                    let float = Value::Float(f64::from_le_bytes(space[start + 1..=start + 8].try_into().unwrap()));
                    start += LightValue::FLOAT_SIZE;
                    float
                },
                LightValue::INTEGER => {
                    let integer = Value::Integer(i64::from_le_bytes(space[start + 1..=start + 8].try_into().unwrap()));
                    start += LightValue::INTEGER_SIZE;
                    integer
                },
                LightValue::STRINGPOINTER => {
                    let index = u32::from_le_bytes(space[start + 1..=start + 4].try_into().unwrap());
                    let string = self.to_string(index as usize);
                    start += LightValue::STRINGPOINTER_SIZE;
                    Value::String(string)
                },
                LightValue::ARRAYPOINTER => {
                    let index = u32::from_le_bytes(space[start + 1..=start + 4].try_into().unwrap());
                    let array = self.to_array(index as usize);
                    start += LightValue::ARRAYPOINTER_SIZE;
                    Value::Array(array)
                },
                _ => unreachable!()
            })
        }
        array
    }

    fn push_into_storage(&mut self, value: Value, stack: &mut [LightValue]) -> LightValue {
        match value {
            Value::Boolean(boolean) => LightValue::Boolean(boolean),
            Value::Nil => LightValue::Nil,
            Value::Float(float) => LightValue::Float(float),
            Value::Integer(integer) => LightValue::Integer(integer),
            Value::String(string) => {
                let index = self.memory.from_space.len();
                let string_bytes = string.into_bytes();
                self.memory.push_to_heap(&string_bytes.len().to_le_bytes(), stack);
                self.memory.push_to_heap(&string_bytes, stack);
                LightValue::StringHeapPointer(index as u32)
            },
            Value::Array(array) => {
                let index = self.memory.from_space.len();
                self.memory.push_to_heap(&0u64.to_le_bytes(), stack);
                for value in array {
                    self.push_into_storage(value, stack);
                };
                let length = self.memory.from_space.len() - index - 8;
                self.memory.from_space[index..index + 8].copy_from_slice(&length.to_le_bytes());
                LightValue::ArrayHeapPointer(index as u32)
            },
            _ => unreachable!()
        }
    }
}

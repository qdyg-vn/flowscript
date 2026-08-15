use crate::builtins::{get_builtin, BuiltinFunction};
use crate::instructions::Bytecode;
use crate::value::{get_kind, LightValue, Value, Kind};
use crate::error_handler::{ErrorHandler, Error, RuntimeError, RuntimeErrorType};
use crate::memory::Memory;

#[derive(Debug)]
pub struct VMConfig {
    pub memory: Memory,
    pub constants_pool: Vec<LightValue>,
    pub heavy_constant_starts: Vec<usize>,
    pub function_starts: Vec<usize>,
}

#[derive(Default)]
pub struct VirMac {
    stations_output: Vec<LightValue>,
    base_pointers: Vec<usize>,
    memory: Memory,
    constants_pool: Vec<LightValue>,
    heavy_constant_starts: Vec<usize>,
    function_starts: Vec<usize>,
    error_handler: ErrorHandler,
}

impl VirMac {
    pub fn new(config: VMConfig, error_handler: ErrorHandler) -> Self {
        Self {
            base_pointers: vec![0],
            memory: config.memory,
            constants_pool: config.constants_pool,
            heavy_constant_starts: config.heavy_constant_starts,
            function_starts: config.function_starts,
            error_handler,
            ..Self::default()
        }
    }

    pub fn execute(&mut self, map: Vec<Vec<u8>>, total_variables: usize) -> Vec<LightValue> {
        let mut stack = vec![LightValue::Nil; 1024];
        for chunk in map {
            self.stations_output.clear();
            let chunk_size = chunk.len();
            let mut stack_position = total_variables;
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
                *instruction_position += Bytecode::LOAD_SIZE;
            },
            Bytecode::LOAD_VARIABLE => {
                let index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                stack[*stack_position] = stack[base_pointer + index as usize];
                self.stations_output.push(stack[*stack_position]);
                *stack_position += 1;
                *instruction_position += Bytecode::LOAD_VARIABLE_SIZE;
            }
            Bytecode::JUMP => {
                let position = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                *instruction_position = position as usize;
            },
            Bytecode::JUMP_IF_FALSE => {
                let position = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                if stack[*stack_position - 1] == LightValue::Boolean(false) {
                    *instruction_position = position as usize;
                } else {
                    *instruction_position += Bytecode::JUMP_IF_FALSE_SIZE;
                };
                *stack_position -= 1;
            }
            Bytecode::STORE => {
                let index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                self.stations_output.push(stack[*stack_position - 1]);
                stack[base_pointer + index as usize] = stack[*stack_position - 1];
                *instruction_position += Bytecode::STORE_SIZE;
            }
            Bytecode::BUILTIN_CALL => {
                let index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                let arity = u16::from_le_bytes([chunk[*instruction_position + 3], chunk[*instruction_position + 4]]);
                let start = *stack_position - arity as usize;
                let end = *stack_position;
                self.execute_builtin_function(index, stack, stack_position, start, end);
                *instruction_position += Bytecode::BUILTIN_CALL_SIZE;
            }
            Bytecode::CALL => {
                let function_index = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]);
                let (functions, arities, variables_counts, lengths) = self.to_function(function_index as usize);
                for index in 0..functions.len() {
                    let (function, arity, variables_count, length) = (&functions[index], arities[index], variables_counts[index], lengths[index]);
                    let mut function_instruction_position = 0;
                    let mut child_stack_position = *stack_position + variables_count as usize;
                    let child_base_pointer = *stack_position - arity as usize;
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
                *instruction_position += Bytecode::CALL_SIZE;
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
                *instruction_position += Bytecode::RETURN_SIZE;
            },
            Bytecode::ARRAY => {
                let count = u32::from_le_bytes(chunk[*instruction_position + 1..=*instruction_position + 4].try_into().unwrap());
                let start = *stack_position - count as usize;
                let mut array = Vec::new();
                for value in stack[start..*stack_position].iter() {
                    match value {
                        LightValue::Boolean(boolean) => {
                            array.push(LightValue::BOOLEAN);
                            array.push(*boolean as u8);
                        },
                        LightValue::Nil => {
                            array.push(LightValue::NIL);
                            array.push(0);
                        },
                        LightValue::Float(float) => {
                            array.push(LightValue::FLOAT);
                            array.extend_from_slice(&float.to_le_bytes());
                        },
                        LightValue::Integer(integer) => {
                            array.push(LightValue::INTEGER);
                            array.extend_from_slice(&integer.to_le_bytes());
                        },
                        LightValue::StringPointer(index) => {
                            array.push(LightValue::STRING_POINTER);
                            array.extend_from_slice(&index.to_le_bytes());
                        },
                        LightValue::ArrayPointer(index) => {
                            array.push(LightValue::ARRAY_POINTER);
                            array.extend_from_slice(&index.to_le_bytes());
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
                *instruction_position += Bytecode::ARRAY_SIZE;
            },
            Bytecode::ADD => {
                let arity = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]) as usize;
                let kind = get_kind(chunk[*instruction_position + 3]);
                let start = *stack_position - arity;
                let (first, rest) = stack[start..*stack_position].split_first().unwrap();
                stack[start] = match kind {
                    Kind::Integer => {
                        rest.iter().fold(*first, |accumulator, x| {
                            match (&accumulator, x) {
                                (LightValue::Integer(a), LightValue::Integer(b)) => LightValue::Integer(a + b),
                                _ => unreachable!()
                            }
                        })
                    },
                    Kind::Float => {
                        rest.iter().fold(*first, |accumulator, x| {
                            match (&accumulator, x) {
                                (LightValue::Float(a), LightValue::Float(b)) => LightValue::Float(a + b),
                                _ => unreachable!()
                            }
                        })
                    },
                    _ => unreachable!()
                };
                *stack_position = start + 1;
                *instruction_position += Bytecode::ADD_SIZE;
            },
            Bytecode::MINUS => {
                let arity = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]) as usize;
                let kind = get_kind(chunk[*instruction_position + 3]);
                let start = *stack_position - arity;
                let (first, rest) = stack[start..*stack_position].split_first().unwrap();
                stack[start] = match kind {
                    Kind::Integer => {
                        rest.iter().fold(*first, |accumulator, x| {
                            match (&accumulator, x) {
                                (LightValue::Integer(a), LightValue::Integer(b)) => LightValue::Integer(a - b),
                                _ => unreachable!()
                            }
                        })
                    },
                    Kind::Float => {
                        rest.iter().fold(*first, |accumulator, x| {
                            match (&accumulator, x) {
                                (LightValue::Float(a), LightValue::Float(b)) => LightValue::Float(a - b),
                                _ => unreachable!()
                            }
                        })
                    },
                    _ => unreachable!()
                };
                *stack_position = start + 1;
                *instruction_position += Bytecode::MINUS_SIZE;
            },
            Bytecode::MULTIPLY => {
                let arity = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]) as usize;
                let kind = get_kind(chunk[*instruction_position + 3]);
                let start = *stack_position - arity;
                let (first, rest) = stack[start..*stack_position].split_first().unwrap();
                stack[start] = match kind {
                    Kind::Integer => {
                        rest.iter().fold(*first, |accumulator, x| {
                            match (&accumulator, x) {
                                (LightValue::Integer(a), LightValue::Integer(b)) => LightValue::Integer(a * b),
                                _ => unreachable!()
                            }
                        })
                    },
                    Kind::Float => {
                        rest.iter().fold(*first, |accumulator, x| {
                            match (&accumulator, x) {
                                (LightValue::Float(a), LightValue::Float(b)) => LightValue::Float(a * b),
                                _ => unreachable!()
                            }
                        })
                    },
                    _ => unreachable!()
                };
                *stack_position = start + 1;
                *instruction_position += Bytecode::MULTIPLY_SIZE;
            },
            Bytecode::EQUAL => {
                let arity = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]) as usize;
                let kind = get_kind(chunk[*instruction_position + 3]);
                let start = *stack_position - arity;
                let (first, rest) = stack[start..*stack_position].split_first().unwrap();
                stack[start] = LightValue::Boolean(match (kind, first) {
                    (Kind::Integer, LightValue::Integer(first_value)) => {
                        rest.iter().all(|x| {
                            match x {
                                LightValue::Integer(a) => a == first_value,
                                _ => unreachable!()
                            }
                        })
                    },
                    (Kind::Float, LightValue::Float(first_value)) => {
                        rest.iter().all(|x| {
                            match x {
                                LightValue::Float(a) => a == first_value,
                                _ => unreachable!()
                            }
                        })
                    },
                    _ => unreachable!()
                });
                *stack_position = start + 1;
                *instruction_position += Bytecode::EQUAL_SIZE;
            },
            Bytecode::LESS_THAN => {
                let arity = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]) as usize;
                let kind = get_kind(chunk[*instruction_position + 3]);
                let start = *stack_position - arity;
                let (first, rest) = stack[start..*stack_position].split_first().unwrap();
                stack[start] = LightValue::Boolean(match (kind, first) {
                    (Kind::Integer, LightValue::Integer(first_value)) => {
                        rest.iter().all(|x| {
                            match x {
                                LightValue::Integer(a) => first_value < a,
                                _ => unreachable!()
                            }
                        })
                    },
                    (Kind::Float, LightValue::Float(first_value)) => {
                        rest.iter().all(|x| {
                            match x {
                                LightValue::Float(a) => first_value < a,
                                _ => unreachable!()
                            }
                        })
                    },
                    _ => unreachable!()
                });
                *stack_position = start + 1;
                *instruction_position += Bytecode::LESS_THAN_SIZE;
            },
            Bytecode::NOT => {
                let LightValue::Boolean(boolean) = stack[*stack_position - 1] else { unreachable!() };
                stack[*stack_position - 1] = LightValue::Boolean(!boolean);
                *instruction_position += Bytecode::NOT_SIZE;
            },
            Bytecode::REVERSE => {
                let arity = u16::from_le_bytes([chunk[*instruction_position + 1], chunk[*instruction_position + 2]]) as usize;
                stack[*stack_position - arity..*stack_position].reverse();
                *instruction_position += Bytecode::REVERSE_SIZE;
            },
            _ => unreachable!()
        }
    }

    fn execute_builtin_function(&mut self, index: u16, stack: &mut [LightValue], stack_position: &mut usize, start: usize, end: usize) {
        let Some(values) = stack.get(start..end) else {
            self.error_handler.fatal(Error::RuntimeError(RuntimeError { kind: RuntimeErrorType::OutOfBounds(start, end) }))
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
            });
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

    fn to_function(&self, index_of_start: usize) -> (Vec<Vec<u8>>, Vec<u8>, Vec<u16>, Vec<usize>) {
        let mut start = self.function_starts[index_of_start];
        let length = u64::from_le_bytes(self.memory.functions[start..start + 8].try_into().unwrap());
        let mut lengths = Vec::with_capacity(length as usize);
        let mut arities = Vec::with_capacity(length as usize);
        let mut variables_counts = Vec::with_capacity(length as usize);
        let mut functions = Vec::with_capacity(length as usize);
        start += 8;
        for _ in 0..length {
            let length = u64::from_le_bytes(self.memory.functions[start..start + 8].try_into().unwrap());
            lengths.push(length as usize);
            let arity = self.memory.functions[start + 8];
            arities.push(arity);
            let variables_count = u16::from_le_bytes([self.memory.functions[start + 9], self.memory.functions[start + 10]]);
            variables_counts.push(variables_count);
            let function = self.memory.functions[start + 11..start + 11 + length as usize].to_vec();
            functions.push(function);
            start += 11 + length as usize;
        }
        (functions, arities, variables_counts, lengths)
    }

    fn to_string(&self, index_of_start: usize) -> String {
        let start = self.heavy_constant_starts[index_of_start];
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
                LightValue::STRING_POINTER => {
                    let index = u32::from_le_bytes(space[start + 1..=start + 4].try_into().unwrap());
                    let string = self.to_string(index as usize);
                    start += LightValue::STRING_POINTER_SIZE;
                    Value::String(string)
                },
                LightValue::ARRAY_POINTER => {
                    let index = u32::from_le_bytes(space[start + 1..=start + 4].try_into().unwrap());
                    let array = self.to_array(index as usize);
                    start += LightValue::ARRAY_POINTER_SIZE;
                    Value::Array(array)
                },
                _ => unreachable!()
            });
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
                let string_length_bytes = string_bytes.len().to_le_bytes();
                self.memory.allocate(string_length_bytes.len() + string_bytes.len(), stack);
                self.memory.push_to_heap(&string_length_bytes);
                self.memory.push_to_heap(&string_bytes);
                LightValue::StringHeapPointer(index as u32)
            },
            Value::Array(array) => {
                let index = self.memory.from_space.len();
                self.memory.push_to_heap(&0u64.to_le_bytes());
                for value in array {
                    self.push_into_storage(value, stack);
                };
                let length = self.memory.from_space.len() - index - 8;
                self.memory.from_space[index..index + 8].copy_from_slice(&length.to_le_bytes());
                LightValue::ArrayHeapPointer(index as u32)
            },
        }
    }
}

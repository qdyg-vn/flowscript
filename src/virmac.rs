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

enum FrameInstruction {
    New(u16),
    Pop,
}

struct CallFrame {
    instruction_position: usize,
    end_instruction: usize,
    stack_position: usize,
    base_pointer: usize,
    reference_pointer: usize,
}

#[derive(Default)]
pub struct VirMac {
    memory: Memory,
    constants_pool: Vec<LightValue>,
    heavy_constant_starts: Vec<usize>,
    function_starts: Vec<usize>,
    error_handler: ErrorHandler,
}

impl VirMac {
    pub fn new(config: VMConfig, error_handler: ErrorHandler) -> Self {
        Self {
            memory: config.memory,
            constants_pool: config.constants_pool,
            heavy_constant_starts: config.heavy_constant_starts,
            function_starts: config.function_starts,
            error_handler,
        }
    }

    pub fn execute(&mut self) -> Vec<LightValue> {
        let mut stack = vec![LightValue::Nil; 1024];
        let start_index = self.function_starts.len() - 1;
        let (function_index, length, variables_count, _arity, max_relative_reference) = self.to_function(start_index);
        let mut frames = Vec::with_capacity(1024);
        frames.push(CallFrame {
            instruction_position: function_index,
            end_instruction: function_index + length,
            stack_position: variables_count as usize + max_relative_reference as usize,
            base_pointer: 0,
            reference_pointer: variables_count as usize,
        });
        while let Some(frame) = frames.last_mut() {
            if frame.instruction_position == frame.end_instruction {
                frames.pop();
                continue
            }
            match self.dispatch_instruction(frame, &mut stack) {
                Some(FrameInstruction::New(start_index)) => {
                    let (function_index, length, variables_count, arity, max_relative_reference) = self.to_function(start_index as usize);
                    let new_frame = CallFrame {
                        instruction_position: function_index,
                        end_instruction: function_index + length,
                        stack_position: frame.stack_position + variables_count as usize + max_relative_reference as usize,
                        base_pointer: frame.stack_position - arity as usize - max_relative_reference as usize,
                        reference_pointer: frame.stack_position - arity as usize,
                    };
                    frames.push(new_frame);
                }
                Some(FrameInstruction::Pop) => { frames.pop(); }
                None => {}
            }
        }
        stack
    }

    fn dispatch_instruction(&mut self, call_frame: &mut CallFrame, stack: &mut Vec<LightValue>) -> Option<FrameInstruction> {
        if stack.len() <= call_frame.stack_position { stack.resize(stack.len() * 2, LightValue::Nil) }
        match self.memory.functions[call_frame.instruction_position] {
            Bytecode::LOAD => {
                let index = u32::from_le_bytes(self.memory.functions[call_frame.instruction_position + 1..=call_frame.instruction_position + 4].try_into().unwrap());
                stack[call_frame.stack_position] = self.constants_pool[index as usize];
                call_frame.stack_position += 1;
                call_frame.instruction_position += Bytecode::LOAD_SIZE;
            },
            Bytecode::LOAD_VARIABLE => {
                let index = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                stack[call_frame.stack_position] = stack[call_frame.base_pointer + index as usize];
                call_frame.stack_position += 1;
                call_frame.instruction_position += Bytecode::LOAD_VARIABLE_SIZE;
            }
            Bytecode::JUMP => {
                let position = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                call_frame.instruction_position = position as usize;
            },
            Bytecode::JUMP_IF_FALSE => {
                let position = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                if stack[call_frame.stack_position - 1] == LightValue::Boolean(false) {
                    call_frame.instruction_position = position as usize;
                } else {
                    call_frame.instruction_position += Bytecode::JUMP_IF_FALSE_SIZE;
                };
                call_frame.stack_position -= 1;
            }
            Bytecode::STORE => {
                let index = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                stack[call_frame.base_pointer + index as usize] = stack[call_frame.stack_position - 1];
                call_frame.instruction_position += Bytecode::STORE_SIZE;
            }
            Bytecode::BUILTIN_CALL => {
                let index = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                let arity = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 3], self.memory.functions[call_frame.instruction_position + 4]]);
                let start = call_frame.stack_position - arity as usize;
                let end = call_frame.stack_position;
                self.execute_builtin_function(index, stack, &mut call_frame.stack_position, start, end);
                call_frame.instruction_position += Bytecode::BUILTIN_CALL_SIZE;
            }
            Bytecode::CALL => {
                let start_index = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                call_frame.instruction_position += Bytecode::CALL_SIZE;
                return Some(FrameInstruction::New(start_index))
            },
            Bytecode::RELATIVE_REFERENCE => {
                let x = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                let y = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 3], self.memory.functions[call_frame.instruction_position + 4]]);
                if y != 0 {
                    todo!("Currently under development")
                }
                stack[call_frame.stack_position] = stack[call_frame.reference_pointer + x as usize];
                call_frame.stack_position += 1;
                call_frame.instruction_position += Bytecode::RELATIVE_REFERENCE_SIZE;
            },
            Bytecode::STATION_CAPTURE => {
                let index = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                stack[call_frame.reference_pointer + index as usize] = stack[call_frame.stack_position - 1];
                call_frame.instruction_position += Bytecode::STATION_CAPTURE_SIZE;
            }
            Bytecode::RETURN => {
                stack.swap(call_frame.base_pointer, call_frame.stack_position - 1);
                return Some(FrameInstruction::Pop)
            },
            Bytecode::ARRAY => {
                let count = u32::from_le_bytes(self.memory.functions[call_frame.instruction_position + 1..=call_frame.instruction_position + 4].try_into().unwrap());
                let start = call_frame.stack_position - count as usize;
                let mut array = Vec::new();
                for value in stack[start..call_frame.stack_position].iter() {
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
                call_frame.stack_position = start + 1;
                call_frame.instruction_position += Bytecode::ARRAY_SIZE;
            },
            Bytecode::ADD => {
                let arity = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]) as usize;
                let kind = get_kind(self.memory.functions[call_frame.instruction_position + 3]);
                let start = call_frame.stack_position - arity;
                let (first, rest) = stack[start..call_frame.stack_position].split_first().unwrap();
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
                call_frame.stack_position = start + 1;
                call_frame.instruction_position += Bytecode::ADD_SIZE;
            },
            Bytecode::MINUS => {
                let arity = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]) as usize;
                let kind = get_kind(self.memory.functions[call_frame.instruction_position + 3]);
                let start = call_frame.stack_position - arity;
                let (first, rest) = stack[start..call_frame.stack_position].split_first().unwrap();
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
                call_frame.stack_position = start + 1;
                call_frame.instruction_position += Bytecode::MINUS_SIZE;
            },
            Bytecode::MULTIPLY => {
                let arity = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]) as usize;
                let kind = get_kind(self.memory.functions[call_frame.instruction_position + 3]);
                let start = call_frame.stack_position - arity;
                let (first, rest) = stack[start..call_frame.stack_position].split_first().unwrap();
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
                call_frame.stack_position = start + 1;
                call_frame.instruction_position += Bytecode::MULTIPLY_SIZE;
            },
            Bytecode::EQUAL => {
                let arity = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]) as usize;
                let kind = get_kind(self.memory.functions[call_frame.instruction_position + 3]);
                let start = call_frame.stack_position - arity;
                let (first, rest) = stack[start..call_frame.stack_position].split_first().unwrap();
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
                call_frame.stack_position = start + 1;
                call_frame.instruction_position += Bytecode::EQUAL_SIZE;
            },
            Bytecode::LESS_THAN => {
                let arity = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]) as usize;
                let kind = get_kind(self.memory.functions[call_frame.instruction_position + 3]);
                let start = call_frame.stack_position - arity;
                let (first, rest) = stack[start..call_frame.stack_position].split_first().unwrap();
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
                call_frame.stack_position = start + 1;
                call_frame.instruction_position += Bytecode::LESS_THAN_SIZE;
            },
            Bytecode::NOT => {
                let LightValue::Boolean(boolean) = stack[call_frame.stack_position - 1] else { unreachable!() };
                stack[call_frame.stack_position - 1] = LightValue::Boolean(!boolean);
                call_frame.instruction_position += Bytecode::NOT_SIZE;
            },
            Bytecode::REVERSE => {
                let arity = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]) as usize;
                stack[call_frame.stack_position - arity..call_frame.stack_position].reverse();
                call_frame.instruction_position += Bytecode::REVERSE_SIZE;
            },
            _ => unreachable!()
        }
        None
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
                        stack[start] = light_value;
                        *stack_position = start + 1;
                    },
                    Err(error) => self.error_handler.fatal(error)
                };
            },
            BuiltinFunction::IO(function) => {
                function(&arguments);
                stack[start] = LightValue::Nil;
                *stack_position = start + 1;
            },
        }
    }

    fn to_function(&self, index_of_start: usize) -> (usize, usize, u16, u8, u8) {
        let start = self.function_starts[index_of_start];
        let length = u64::from_le_bytes(self.memory.functions[start..start + 8].try_into().unwrap());
        let variables_count = u16::from_le_bytes([self.memory.functions[start + 8], self.memory.functions[start + 9]]);
        let arity = self.memory.functions[start + 10];
        let max_relative_reference = self.memory.functions[start + 11];
        (start + 12, length as usize, variables_count, arity, max_relative_reference)
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

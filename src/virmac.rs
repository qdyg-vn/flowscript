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
        let (function_index, length, variables_count, _arity, max_relative_reference) = self.get_function(start_index);
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
                    let (function_index, length, variables_count, arity, max_relative_reference) = self.get_function(start_index as usize);
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
                    self.push_into_array(self.light_value_to_value(*value), &mut array)
                }
                let index = self.memory.from_space.len();
                self.memory.from_space.extend_from_slice(&array.len().to_le_bytes());
                self.memory.from_space.extend_from_slice(&array);
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
            arguments.push(self.light_value_to_value(*argument));
        }
        let builtin = get_builtin(index);
        match builtin.function {
            BuiltinFunction::Math(function) | BuiltinFunction::Collection(function) | BuiltinFunction::Compare(function) | BuiltinFunction::Casting(function) => {
                match function(arguments) {
                    Ok(value) => {
                        let light_value = self.push_into_stack(value, stack);
                        stack[start] = light_value;
                        *stack_position = start + 1;
                    },
                    Err(error) => self.error_handler.fatal(error)
                };
            },
            BuiltinFunction::IO(function) => {
                function(arguments);
                stack[start] = LightValue::Nil;
                *stack_position = start + 1;
            },
        }
    }

    fn get_function(&self, index_of_start: usize) -> (usize, usize, u16, u8, u8) {
        let start = self.function_starts[index_of_start];
        let length = u64::from_le_bytes(self.memory.functions[start..start + 8].try_into().unwrap());
        let variables_count = u16::from_le_bytes([self.memory.functions[start + 8], self.memory.functions[start + 9]]);
        let arity = self.memory.functions[start + 10];
        let max_relative_reference = self.memory.functions[start + 11];
        (start + 12, length as usize, variables_count, arity, max_relative_reference)
    }

    fn get_string_in_permanent_space(&self, index_of_start: usize) -> String {
        let start = self.heavy_constant_starts[index_of_start];
        let length = u64::from_le_bytes(self.memory.permanent_space[start..start + 8].try_into().unwrap());
        String::from_utf8(self.memory.permanent_space[start + 8..start + 8 + length as usize].to_vec()).unwrap()
    }

    fn get_string_in_heap(&self, start: usize) -> String {
        let length = u64::from_le_bytes(self.memory.from_space[start..start + 8].try_into().unwrap());
        String::from_utf8(self.memory.from_space[start + 8..start + 8 + length as usize].to_vec()).unwrap()
    }

    fn get_array(&self, start: &mut usize) -> Vec<Value> {
        let length = u64::from_le_bytes(self.memory.from_space[*start..*start + 8].try_into().unwrap());
        *start += 8;
        let end = *start + length as usize;
        let mut array = Vec::new();
        while *start < end {
            array.push(match self.memory.from_space[*start] {
                Kind::BOOLEAN => {
                    let boolean = Value::Boolean(self.memory.from_space[*start] != 0);
                    *start += LightValue::BOOLEAN_SIZE;
                    boolean
                },
                Kind::NIL => {
                    *start += LightValue::NIL_SIZE;
                    Value::Nil
                },
                Kind::FLOAT => {
                    let float = Value::Float(f64::from_le_bytes(self.memory.from_space[*start + 1..=*start + 8].try_into().unwrap()));
                    *start += LightValue::FLOAT_SIZE;
                    float
                },
                Kind::INTEGER => {
                    let integer = Value::Integer(i64::from_le_bytes(self.memory.from_space[*start + 1..=*start + 8].try_into().unwrap()));
                    *start += LightValue::INTEGER_SIZE;
                    integer
                },
                Kind::STRING => {
                    *start += 1;
                    let length = u64::from_le_bytes(self.memory.from_space[*start..*start + 8].try_into().unwrap());
                    *start += 8;
                    let string = String::from_utf8(self.memory.from_space[*start..*start + length as usize].to_vec()).unwrap();
                    *start += length as usize;
                    Value::String(string)
                },
                Kind::ARRAY => {
                    *start += 1;
                    let array = self.get_array(start);
                    Value::Array(array)
                },
                _ => unreachable!()
            });
        }
        array
    }

    fn push_into_stack(&mut self, value: Value, stack: &mut [LightValue]) -> LightValue {
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
                let mut byte_array = Vec::new();
                array.into_iter().for_each(|element| self.push_into_array(element, &mut byte_array));
                let array_length_bytes = byte_array.len().to_le_bytes();
                self.memory.allocate(array_length_bytes.len() + byte_array.len(), stack);
                self.memory.push_to_heap(&array_length_bytes);
                self.memory.push_to_heap(&byte_array);
                LightValue::ArrayPointer(index as u32)
            },
        }
    }

    fn push_into_array(&mut self, value: Value, target_array: &mut Vec<u8>) {
        match value {
            Value::Boolean(boolean) => {
                target_array.push(Kind::BOOLEAN);
                target_array.push(boolean as u8);
            },
            Value::Nil => {
                target_array.push(Kind::NIL);
            },
            Value::Float(float) => {
                target_array.push(Kind::FLOAT);
                target_array.extend_from_slice(&float.to_le_bytes());
            },
            Value::Integer(integer) => {
                target_array.push(Kind::INTEGER);
                target_array.extend_from_slice(&integer.to_le_bytes());
            },
            Value::String(string) => {
                target_array.push(Kind::STRING);
                let string_bytes = string.into_bytes();
                let string_length_bytes = string_bytes.len().to_le_bytes();
                target_array.extend_from_slice(&string_length_bytes);
                target_array.extend_from_slice(&string_bytes);
            },
            Value::Array(array) => {
                let mut byte_array = Vec::new();
                array.into_iter().for_each(|element| self.push_into_array(element, &mut byte_array));
                target_array.push(Kind::ARRAY);
                target_array.extend_from_slice(&byte_array.len().to_le_bytes());
                target_array.extend_from_slice(&byte_array);
            },
        }
    }

    fn light_value_to_value(&self, light_value: LightValue) -> Value {
        match light_value {
            LightValue::Boolean(boolean) => Value::Boolean(boolean),
            LightValue::Nil => Value::Nil,
            LightValue::Float(float) => Value::Float(float),
            LightValue::Integer(integer) => Value::Integer(integer),
            LightValue::StringPointer(index) => {
                let string = self.get_string_in_permanent_space(index as usize);
                Value::String(string)
            },
            LightValue::StringHeapPointer(index) => {
                let string = self.get_string_in_heap(index as usize);
                Value::String(string)
            },
            LightValue::ArrayPointer(index) => Value::Array(self.get_array(&mut (index as usize))),
        }
    }
}

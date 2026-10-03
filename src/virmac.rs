use crate::builtins::{get_builtin, BuiltinFunction};
use crate::instructions::Bytecode;
use crate::value::{get_kind, LightValue, Value, Kind, Tag, BOOLEAN_SIZE, FLOAT_SIZE, INTEGER_SIZE, INDEX_SIZE};
use crate::error_handler::{ErrorHandler, Error, RuntimeError, RuntimeErrorType};
use crate::memory::Memory;

#[derive(Debug)]
pub struct VMConfig {
    pub memory: Memory,
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
    temp_pointer: usize,
    reference_pointer: usize,
}

#[derive(Default)]
pub struct VirMac {
    memory: Memory,
    function_starts: Vec<usize>,
    error_handler: ErrorHandler,
}

impl VirMac {
    pub fn new(config: VMConfig, error_handler: ErrorHandler) -> Self {
        Self {
            memory: config.memory,
            function_starts: config.function_starts,
            error_handler,
        }
    }

    pub fn execute(&mut self) -> Vec<LightValue> {
        let mut stack = vec![LightValue::Nil; 1024];
        let start_index = self.function_starts.len() - 1;
        let (function_index, length, variables_count, _arity, max_temp_variables, max_relative_reference) = self.get_function(start_index);
        let mut frames = Vec::with_capacity(1024);
        frames.push(CallFrame {
            instruction_position: function_index,
            end_instruction: function_index + length,
            stack_position: variables_count as usize + max_temp_variables as usize + max_relative_reference as usize,
            base_pointer: 0,
            temp_pointer: variables_count as usize,
            reference_pointer: variables_count as usize + max_temp_variables as usize,
        });
        while let Some(frame) = frames.last_mut() {
            if frame.instruction_position == frame.end_instruction {
                frames.pop();
                continue
            }
            match self.dispatch_instruction(frame, &mut stack) {
                Some(FrameInstruction::New(start_index)) => {
                    let (function_index, length, variables_count, arity, max_temp_variables, max_relative_reference) = self.get_function(start_index as usize);
                    let new_frame = CallFrame {
                        instruction_position: function_index,
                        end_instruction: function_index + length,
                        stack_position: frame.stack_position + variables_count as usize + max_temp_variables as usize + max_relative_reference as usize,
                        base_pointer: frame.stack_position - arity as usize,
                        temp_pointer: frame.stack_position + variables_count as usize,
                        reference_pointer: frame.stack_position + variables_count as usize + max_temp_variables as usize,
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
                let tag = self.memory.functions[call_frame.instruction_position + 1];
                stack[call_frame.stack_position] = match tag {
                    Tag::BOOLEAN => {
                        let value = LightValue::Boolean(self.memory.functions[call_frame.instruction_position + 2] != 0);
                        call_frame.instruction_position += BOOLEAN_SIZE;
                        value
                    },
                    Tag::NIL => LightValue::Nil,
                    Tag::FLOAT => {
                        let value = LightValue::Float(f64::from_le_bytes(self.memory.functions[call_frame.instruction_position + 2..=call_frame.instruction_position + 9].try_into().unwrap()));
                        call_frame.instruction_position += FLOAT_SIZE;
                        value
                    },
                    Tag::INTEGER => {
                        let value = LightValue::Integer(i64::from_le_bytes(self.memory.functions[call_frame.instruction_position + 2..=call_frame.instruction_position + 9].try_into().unwrap()));
                        call_frame.instruction_position += INTEGER_SIZE;
                        value
                    },
                    Tag::STRING => {
                        let value = LightValue::StringPointer(u32::from_le_bytes(self.memory.functions[call_frame.instruction_position + 2..=call_frame.instruction_position + 5].try_into().unwrap()));
                        call_frame.instruction_position += INDEX_SIZE;
                        value
                    },
                    _ => unreachable!()
                };
                call_frame.stack_position += 1;
                call_frame.instruction_position += Bytecode::LOAD_SIZE;
            },
            Bytecode::LOAD_VARIABLE => {
                let index = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                stack[call_frame.stack_position] = stack[call_frame.base_pointer + index as usize];
                call_frame.stack_position += 1;
                call_frame.instruction_position += Bytecode::LOAD_VARIABLE_SIZE;
            },
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
            },
            Bytecode::STORE => {
                let index = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                stack[call_frame.base_pointer + index as usize] = stack[call_frame.stack_position - 1];
                call_frame.instruction_position += Bytecode::STORE_SIZE;
            },
            Bytecode::STORE_TEMP => {
                let temp_index = self.memory.functions[call_frame.instruction_position + 1];
                stack[call_frame.temp_pointer + temp_index as usize] = stack[call_frame.stack_position - 1];
                call_frame.instruction_position += Bytecode::STORE_TEMP_SIZE;
            },
            Bytecode::BUILTIN_CALL => {
                let index = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                let arity = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 3], self.memory.functions[call_frame.instruction_position + 4]]);
                let start = call_frame.stack_position - arity as usize;
                let end = call_frame.stack_position;
                self.execute_builtin_function(index, stack, &mut call_frame.stack_position, start, end);
                call_frame.instruction_position += Bytecode::BUILTIN_CALL_SIZE;
            },
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
            },
            Bytecode::RETURN => {
                stack.swap(call_frame.base_pointer, call_frame.stack_position - 1);
                return Some(FrameInstruction::Pop)
            },
            Bytecode::ARRAY => {
                let count = u32::from_le_bytes(self.memory.functions[call_frame.instruction_position + 1..=call_frame.instruction_position + 4].try_into().unwrap());
                let start = call_frame.stack_position - count as usize;
                let mut array = Vec::new();
                self.push_light_value_into_array(&stack[start..call_frame.stack_position], &mut array);
                let index = self.memory.from_space.len();
                let array_length_bytes = array.len().to_le_bytes();
                self.memory.allocate(array_length_bytes.len() + array.len(), stack);
                self.memory.push_to_heap(&array_length_bytes);
                self.memory.push_to_heap(&array);
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
            Bytecode::STRUCT_INSTANTIATION => {
                let fields_count = self.memory.functions[call_frame.instruction_position + 1];
                let index = self.memory.from_space.len();
                let mut structure = Vec::new();
                self.push_light_value_into_array(&stack[call_frame.temp_pointer..call_frame.temp_pointer + fields_count as usize], &mut structure);
                let struct_length_bytes = structure.len().to_le_bytes();
                self.memory.allocate(struct_length_bytes.len() + structure.len(), stack);
                self.memory.push_to_heap(&struct_length_bytes);
                self.memory.push_to_heap(&structure);
                stack[call_frame.stack_position] = LightValue::StructPointer(index as u32);
                call_frame.stack_position += 1;
                call_frame.instruction_position += Bytecode::STRUCT_INSTANTIATION_SIZE;
            },
            Bytecode::FIELD_ACCESS => {
                let index = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 1], self.memory.functions[call_frame.instruction_position + 2]]);
                let field_index = u16::from_le_bytes([self.memory.functions[call_frame.instruction_position + 3], self.memory.functions[call_frame.instruction_position + 4]]);
                let LightValue::StructPointer(struct_index) = stack[call_frame.base_pointer + index as usize] else { unreachable!() };
                let light_value = self.get_field_value(&mut (struct_index as usize), field_index as usize);
                stack[call_frame.stack_position] = light_value;
                call_frame.stack_position += 1;
                call_frame.instruction_position += Bytecode::FIELD_ACCESS_SIZE;
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
                        let light_value = self.value_to_light_value(value, stack);
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

    fn get_function(&self, index_of_start: usize) -> (usize, usize, u16, u8, u8, u8) {
        let start = self.function_starts[index_of_start];
        let length = u64::from_le_bytes(self.memory.functions[start..start + 8].try_into().unwrap());
        let variables_count = u16::from_le_bytes([self.memory.functions[start + 8], self.memory.functions[start + 9]]);
        let arity = self.memory.functions[start + 10];
        let max_temp_variables = self.memory.functions[start + 11];
        let max_relative_reference = self.memory.functions[start + 12];
        (start + 13, length as usize, variables_count, arity, max_temp_variables, max_relative_reference)
    }

    fn get_string_in_permanent_space(&self, start: usize) -> String {
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
                Tag::BOOLEAN => {
                    let boolean = Value::Boolean(self.memory.from_space[*start + 1] != 0);
                    *start += LightValue::BOOLEAN_SIZE;
                    boolean
                },
                Tag::NIL => {
                    *start += LightValue::NIL_SIZE;
                    Value::Nil
                },
                Tag::FLOAT => {
                    let float = Value::Float(f64::from_le_bytes(self.memory.from_space[*start + 1..=*start + 8].try_into().unwrap()));
                    *start += LightValue::FLOAT_SIZE;
                    float
                },
                Tag::INTEGER => {
                    let integer = Value::Integer(i64::from_le_bytes(self.memory.from_space[*start + 1..=*start + 8].try_into().unwrap()));
                    *start += LightValue::INTEGER_SIZE;
                    integer
                },
                Tag::STRING => {
                    *start += 1;
                    let length = u64::from_le_bytes(self.memory.from_space[*start..*start + 8].try_into().unwrap());
                    *start += 8;
                    let string = String::from_utf8(self.memory.from_space[*start..*start + length as usize].to_vec()).unwrap();
                    *start += length as usize;
                    Value::String(string)
                },
                Tag::STRING_POINTER => {
                    let index = u32::from_le_bytes(self.memory.from_space[*start + 1..=*start + 4].try_into().unwrap());
                    let string = self.get_string_in_permanent_space(index as usize);
                    Value::String(string)
                },
                Tag::STRING_HEAP_POINTER => {
                    let index = u32::from_le_bytes(self.memory.from_space[*start + 1..=*start + 4].try_into().unwrap());
                    let string = self.get_string_in_heap(index as usize);
                    Value::String(string)
                },
                Tag::ARRAY => {
                    *start += 1;
                    let array = self.get_array(start);
                    Value::Array(array)
                },
                Tag::ARRAY_POINTER => {
                    let index = u32::from_le_bytes(self.memory.from_space[*start + 1..=*start + 4].try_into().unwrap());
                    Value::Array(self.get_array(&mut (index as usize)))
                },
                Tag::STRUCT => {
                    *start += 1;
                    let structure = self.get_array(start);
                    Value::Struct(structure)
                },
                Tag::STRUCT_POINTER => {
                    let index = u32::from_le_bytes(self.memory.from_space[*start + 1..=*start + 4].try_into().unwrap());
                    Value::Struct(self.get_array(&mut (index as usize)))
                },
                _ => unreachable!()
            });
        }
        array
    }

    fn get_field_value(&self, start: &mut usize, field_index: usize) -> LightValue {
        *start += 8 + field_index;
        match self.memory.from_space[*start] {
            Tag::BOOLEAN => LightValue::Boolean(self.memory.from_space[*start + 1] != 0),
            Tag::FLOAT => LightValue::Float(f64::from_le_bytes(self.memory.from_space[*start + 1..=*start + 8].try_into().unwrap())),
            Tag::INTEGER => LightValue::Integer(i64::from_le_bytes(self.memory.from_space[*start + 1..=*start + 8].try_into().unwrap())),
            Tag::STRING_POINTER => LightValue::StringPointer(u32::from_le_bytes(self.memory.from_space[*start + 1..=*start + 4].try_into().unwrap())),
            Tag::STRING_HEAP_POINTER => LightValue::StringHeapPointer(u32::from_le_bytes(self.memory.from_space[*start + 1..=*start + 4].try_into().unwrap())),
            Tag::ARRAY_POINTER => LightValue::ArrayPointer(u32::from_le_bytes(self.memory.from_space[*start + 1..=*start + 4].try_into().unwrap())),
            Tag::STRUCT_POINTER => LightValue::StructPointer(u32::from_le_bytes(self.memory.from_space[*start + 1..=*start + 4].try_into().unwrap())),
            _ => unreachable!()
        }
    }

    fn value_to_light_value(&mut self, value: Value, stack: &mut [LightValue]) -> LightValue {
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
                self.push_value_into_array(array, &mut byte_array);
                let array_length_bytes = byte_array.len().to_le_bytes();
                self.memory.allocate(array_length_bytes.len() + byte_array.len(), stack);
                self.memory.push_to_heap(&array_length_bytes);
                self.memory.push_to_heap(&byte_array);
                LightValue::ArrayPointer(index as u32)
            },
            Value::Struct(fields) => {
                let index = self.memory.from_space.len();
                let mut structure = Vec::new();
                self.push_value_into_array(fields, &mut structure);
                let struct_length_bytes = structure.len().to_le_bytes();
                self.memory.allocate(struct_length_bytes.len() + structure.len(), stack);
                self.memory.push_to_heap(&struct_length_bytes);
                self.memory.push_to_heap(&structure);
                LightValue::StructPointer(index as u32)
            }
        }
    }

    fn push_light_value_into_array(&mut self, elements: &[LightValue], target_array: &mut Vec<u8>) {
        for element in elements {
            match element {
                LightValue::Boolean(boolean) => {
                    target_array.push(Tag::BOOLEAN);
                    target_array.push(*boolean as u8);
                },
                LightValue::Nil => {
                    target_array.push(Tag::NIL);
                },
                LightValue::Float(float) => {
                    target_array.push(Tag::FLOAT);
                    target_array.extend_from_slice(&float.to_le_bytes());
                },
                LightValue::Integer(integer) => {
                    target_array.push(Tag::INTEGER);
                    target_array.extend_from_slice(&integer.to_le_bytes());
                },
                LightValue::ArrayPointer(index) => {
                    target_array.push(Tag::ARRAY_POINTER);
                    target_array.extend_from_slice(&index.to_le_bytes());
                },
                LightValue::StringPointer(index) => {
                    target_array.push(Tag::STRING_POINTER);
                    target_array.extend_from_slice(&index.to_le_bytes());
                },
                LightValue::StringHeapPointer(index) => {
                    target_array.push(Tag::STRING_HEAP_POINTER);
                    target_array.extend_from_slice(&index.to_le_bytes());
                },
                LightValue::StructPointer(index) => {
                    target_array.push(Tag::STRUCT_POINTER);
                    target_array.extend_from_slice(&index.to_le_bytes());
                },
            }
        }
    }

    fn push_value_into_array(&mut self, elements: Vec<Value>, target_array: &mut Vec<u8>) {
        for element in elements {
            match element {
                Value::Boolean(boolean) => {
                    target_array.push(Tag::BOOLEAN);
                    target_array.push(boolean as u8);
                },
                Value::Nil => target_array.push(Tag::NIL),
                Value::Float(float) => {
                    target_array.push(Tag::FLOAT);
                    target_array.extend_from_slice(&float.to_le_bytes());
                },
                Value::Integer(integer) => {
                    target_array.push(Tag::INTEGER);
                    target_array.extend_from_slice(&integer.to_le_bytes());
                },
                Value::String(string) => {
                    target_array.push(Tag::STRING);
                    let string_bytes = string.into_bytes();
                    let string_length_bytes = string_bytes.len().to_le_bytes();
                    target_array.extend_from_slice(&string_length_bytes);
                    target_array.extend_from_slice(&string_bytes);
                },
                Value::Array(elements) => {
                    let mut array = Vec::new();
                    self.push_value_into_array(elements, &mut array);
                    target_array.push(Tag::ARRAY);
                    target_array.extend_from_slice(&array.len().to_le_bytes());
                    target_array.extend_from_slice(&array);
                },
                Value::Struct(fields) => {
                    let mut structure = Vec::new();
                    self.push_value_into_array(fields, &mut structure);
                    target_array.push(Tag::STRUCT);
                    target_array.extend_from_slice(&structure.len().to_le_bytes());
                    target_array.extend_from_slice(&structure);
                }
            }
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
            LightValue::StructPointer(index) => Value::Struct(self.get_array(&mut (index as usize))),
        }
    }
}

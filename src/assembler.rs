use crate::instructions::{Bytecode, Instruction};
use crate::memory::Memory;
use crate::constants_pool::ConstantsPool;
use crate::value::{LightValue, Value, BOOLEAN_SIZE, FLOAT_SIZE, INTEGER_SIZE, INDEX_SIZE, NIL_SIZE};
use crate::virmac::VMConfig;

#[derive(Default)]
pub struct Assembler {
    memory: Memory,
    constants_pool: ConstantsPool,
    heavy_constants_offset: Vec<u32>,
}

impl Assembler {
    pub fn new(memory: Memory, constants_pool: ConstantsPool) -> Self {
        Self {
            memory,
            constants_pool,
            ..Self::default()
        }
    }

    pub fn assemble_map(mut self) -> VMConfig {
        self.assemble_heavy_constants();
        let main_function_index = self.assemble_function() as usize;
        VMConfig {
            memory: self.memory,
            main_function_index
        }
    }

    fn assemble_heavy_constants(&mut self) {
        for constant in std::mem::take(&mut self.constants_pool.heavy_constants) {
            match constant {
                Value::String(string) => {
                    self.heavy_constants_offset.push(self.memory.permanent_space.len() as u32);
                    let string_bytes = string.into_bytes();
                    self.memory.permanent_space.extend_from_slice(&string_bytes.len().to_le_bytes());
                    self.memory.permanent_space.extend_from_slice(&string_bytes)
                },
                _ => unreachable!()
            }
        };
    }

    fn assemble_function(&mut self) -> u32 {
        let mut function_offsets = Vec::new();
        let mut call_instructions_position = Vec::new();
        for function in std::mem::take(&mut self.constants_pool.functions) {
            function_offsets.push(self.memory.functions.len() as u32);
            let variables_count_length = 2;
            let arity_length = 1;
            let max_temp_variables_length = 1;
            let max_relative_reference_length = 1;
            let length_length = 8;
            let mut byte_position = self.memory.functions.len() + variables_count_length + arity_length + max_temp_variables_length + max_relative_reference_length + length_length;
            let mut byte_chunk = function.variables_count.to_le_bytes().to_vec();
            byte_chunk.extend_from_slice(&[function.arity, function.max_temp_variables, function.max_relative_reference]);
            self.assemble_instruction(function.instructions, &mut byte_position, &mut byte_chunk, &mut call_instructions_position);
            self.memory.functions.extend_from_slice(&(byte_chunk.len() - variables_count_length - arity_length - max_temp_variables_length - max_relative_reference_length).to_le_bytes());
            self.memory.functions.extend_from_slice(&byte_chunk)
        };
        for position in call_instructions_position {
            let function_index = u32::from_le_bytes(self.memory.functions[position as usize + 1..=position as usize + 4].try_into().unwrap());
            let function_offset = function_offsets[function_index as usize].to_le_bytes();
            self.memory.functions[position as usize + 1..=position as usize + 4].copy_from_slice(&function_offset);
        }
        *function_offsets.last().unwrap()
    }

    fn assemble_instruction(&mut self, instructions: Vec<Instruction>, byte_position: &mut usize, byte_chunk: &mut Vec<u8>, call_instructions_position: &mut Vec<u32>) {
        let chunk_size = instructions.len();
        let mut position = 0;
        while position < chunk_size {
            match instructions[position] {
                Instruction::Load(value) => {
                    byte_chunk.push(Bytecode::Load as u8);
                    byte_chunk.push(value.get_kind().to_byte());
                    match value {
                        LightValue::Boolean(boolean) => {
                            byte_chunk.push(boolean as u8);
                            *byte_position += BOOLEAN_SIZE;
                        },
                        LightValue::Nil => {},
                        LightValue::Float(float) => {
                            byte_chunk.extend_from_slice(&float.to_le_bytes());
                            *byte_position += FLOAT_SIZE;
                        },
                        LightValue::Integer(integer) => {
                            byte_chunk.extend_from_slice(&integer.to_le_bytes());
                            *byte_position += INTEGER_SIZE;
                        },
                        _ => unreachable!(),
                    };
                    *byte_position += Bytecode::LOAD_SIZE
                },
                Instruction::HeavyLoad(index, kind) => {
                    byte_chunk.push(Bytecode::Load as u8);
                    byte_chunk.push(kind.to_byte());
                    byte_chunk.extend_from_slice(&self.heavy_constants_offset[index as usize].to_le_bytes());
                    *byte_position += Bytecode::LOAD_SIZE;
                },
                Instruction::BuiltinCall(index, arity) => {
                    byte_chunk.push(Bytecode::BuiltinCall as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    *byte_position += Bytecode::BUILTIN_CALL_SIZE
                },
                Instruction::Call(index) => {
                    call_instructions_position.push(*byte_position as u32);
                    byte_chunk.push(Bytecode::Call as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    *byte_position += Bytecode::CALL_SIZE
                },
                Instruction::RelativeReference(x, y) => {
                    byte_chunk.push(Bytecode::RelativeReference as u8);
                    byte_chunk.extend_from_slice(&x.to_le_bytes());
                    byte_chunk.extend_from_slice(&y.to_le_bytes());
                    *byte_position += Bytecode::RELATIVE_REFERENCE_SIZE
                },
                Instruction::Store(index) => {
                    byte_chunk.push(Bytecode::Store as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    *byte_position += Bytecode::STORE_SIZE
                },
                Instruction::StoreTemp(index) => {
                    byte_chunk.push(Bytecode::StoreTemp as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    *byte_position += Bytecode::STORE_TEMP_SIZE
                },
                Instruction::StationCapture(index) => {
                    byte_chunk.push(Bytecode::StationCapture as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    *byte_position += Bytecode::STATION_CAPTURE_SIZE
                },
                Instruction::LoadVariable(index) => {
                    byte_chunk.push(Bytecode::LoadVariable as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    *byte_position += Bytecode::LOAD_VARIABLE_SIZE
                },
                Instruction::Return => {
                    byte_chunk.push(Bytecode::Return as u8);
                    *byte_position += Bytecode::RETURN_SIZE
                },
                Instruction::Array(count) => {
                    byte_chunk.push(Bytecode::Array as u8);
                    byte_chunk.extend_from_slice(&count.to_le_bytes());
                    *byte_position += Bytecode::ARRAY_SIZE
                },
                Instruction::Jump(index) => {
                    let instruction_byte_length = Bytecode::JUMP_SIZE;
                    byte_chunk.push(Bytecode::Jump as u8);
                    byte_chunk.extend_from_slice(&((*byte_position + instruction_byte_length + self.get_byte_distance(&instructions, position + 1, index)) as u16).to_le_bytes());
                    *byte_position += Bytecode::JUMP_SIZE
                },
                Instruction::JumpIfFalse(index) => {
                    let instruction_byte_length = Bytecode::JUMP_IF_FALSE_SIZE;
                    byte_chunk.push(Bytecode::JumpIfFalse as u8);
                    byte_chunk.extend_from_slice(&((*byte_position + instruction_byte_length + self.get_byte_distance(&instructions, position + 1, index)) as u16).to_le_bytes());
                    *byte_position += Bytecode::JUMP_IF_FALSE_SIZE
                },
                Instruction::Not => {
                    byte_chunk.push(Bytecode::Not as u8);
                    *byte_position += Bytecode::NOT_SIZE
                },
                Instruction::Reverse(arity) => {
                    byte_chunk.push(Bytecode::Reverse as u8);
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    *byte_position += Bytecode::REVERSE_SIZE
                },
                Instruction::Add(arity, kind) => {
                    byte_chunk.push(Bytecode::Add as u8);
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    byte_chunk.push(kind.to_byte());
                    *byte_position += Bytecode::ADD_SIZE
                }
                Instruction::Minus(arity, kind) => {
                    byte_chunk.push(Bytecode::Minus as u8);
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    byte_chunk.push(kind.to_byte());
                    *byte_position += Bytecode::MINUS_SIZE
                },
                Instruction::Multiply(arity, kind) => {
                    byte_chunk.push(Bytecode::Multiply as u8);
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    byte_chunk.push(kind.to_byte());
                    *byte_position += Bytecode::MULTIPLY_SIZE
                },
                Instruction::Equal(arity, kind) => {
                    byte_chunk.push(Bytecode::Equal as u8);
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    byte_chunk.push(kind.to_byte());
                    *byte_position += Bytecode::EQUAL_SIZE
                },
                Instruction::LessThan(arity, kind) => {
                    byte_chunk.push(Bytecode::LessThan as u8);
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    byte_chunk.push(kind.to_byte());
                    *byte_position += Bytecode::LESS_THAN_SIZE
                },
                Instruction::StructInstantiation(fields_count) => {
                    byte_chunk.push(Bytecode::StructInstantiation as u8);
                    byte_chunk.extend_from_slice(&fields_count.to_le_bytes());
                    *byte_position += Bytecode::STRUCT_INSTANTIATION_SIZE
                },
                Instruction::FieldAccess(index, field_offset) => {
                    byte_chunk.push(Bytecode::FieldAccess as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    byte_chunk.extend_from_slice(&field_offset.to_le_bytes());
                    *byte_position += Bytecode::FIELD_ACCESS_SIZE
                },
            }
            position += 1
        }
    }

    fn get_byte_distance(&self, instructions: &[Instruction], mut position: usize, target: u16) -> usize {
        let mut distance = 0;
        while position < target as usize {
            match instructions[position] {
                Instruction::Call(_) => distance += Bytecode::CALL_SIZE,
                Instruction::BuiltinCall(_, _) => distance += Bytecode::BUILTIN_CALL_SIZE,
                Instruction::Load(value) => distance += Bytecode::LOAD_SIZE + match value {
                    LightValue::Boolean(_) => BOOLEAN_SIZE,
                    LightValue::Nil => NIL_SIZE,
                    LightValue::Float(_) => FLOAT_SIZE,
                    LightValue::Integer(_) => INTEGER_SIZE,
                    _ => unreachable!()
                },
                Instruction::HeavyLoad(_, _) => distance += Bytecode::LOAD_SIZE + INDEX_SIZE,
                Instruction::LoadVariable(_) => distance += Bytecode::LOAD_VARIABLE_SIZE,
                Instruction::Jump(_) => distance += Bytecode::JUMP_SIZE,
                Instruction::JumpIfFalse(_) => distance += Bytecode::JUMP_IF_FALSE_SIZE,
                Instruction::RelativeReference(_, _) => distance += Bytecode::RELATIVE_REFERENCE_SIZE,
                Instruction::Return => distance += Bytecode::RETURN_SIZE,
                Instruction::Store(_) => distance += Bytecode::STORE_SIZE,
                Instruction::StoreTemp(_) => distance += Bytecode::STORE_TEMP_SIZE,
                Instruction::StationCapture(_) => distance += Bytecode::STATION_CAPTURE_SIZE,
                Instruction::Array(_) => distance += Bytecode::ARRAY_SIZE,
                Instruction::Not => distance += Bytecode::NOT_SIZE,
                Instruction::Reverse(_) => distance += Bytecode::REVERSE_SIZE,
                Instruction::Add(_, _) => distance += Bytecode::ADD_SIZE,
                Instruction::Minus(_, _) => distance += Bytecode::MINUS_SIZE,
                Instruction::Multiply(_, _) => distance += Bytecode::MULTIPLY_SIZE,
                Instruction::Equal(_, _) => distance += Bytecode::EQUAL_SIZE,
                Instruction::LessThan(_, _) => distance += Bytecode::LESS_THAN_SIZE,
                Instruction::StructInstantiation(_) => distance += Bytecode::STRUCT_INSTANTIATION_SIZE,
                Instruction::FieldAccess(_, _) => distance += Bytecode::FIELD_ACCESS_SIZE,
            }
            position += 1
        }
        distance
    }
}

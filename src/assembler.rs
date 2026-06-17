use crate::instructions::{Bytecode, Instruction, Chunk};
use crate::memory::Memory;
use crate::constants_pool::ConstantsPool;
use crate::value::{LightValue, HeavyValue};

pub struct Assembler {
    memory: Memory,
    constants_pool: ConstantsPool,
}

impl Assembler {
    pub fn new(memory: Memory, constants_pool: ConstantsPool) -> Self {
        Self {
            memory,
            constants_pool,
        }
    }

    pub fn assemble_map(mut self, map: Vec<Chunk>) -> (Vec<Vec<u8>>, Vec<usize>, Vec<LightValue>, Memory) {
        let mut byte_map = Vec::new();
        for chunk in map {
            let mut byte_position = 0;
            let mut byte_chunk = Vec::new();
            self.assemble_instruction(chunk.instructions, &mut byte_position, &mut byte_chunk);
            byte_map.push(byte_chunk)
        }
        (byte_map, self.assemble_heavy_constants(), self.constants_pool.constants, self.memory)
    }

    fn assemble_heavy_constants(&mut self) -> Vec<usize> {
        let mut starts = Vec::new();
        for constant in std::mem::take(&mut self.constants_pool.heavy_constants) {
            match constant {
                HeavyValue::String(string) => {
                    starts.push(self.memory.permanent_space.len());
                    let string_bytes = string.into_bytes();
                    self.memory.permanent_space.extend_from_slice(&string_bytes.len().to_le_bytes());
                    self.memory.permanent_space.extend_from_slice(&string_bytes)
                },
                HeavyValue::Function(body) => {
                    starts.push(self.memory.permanent_space.len());
                    let mut byte_position = 0;
                    let mut byte_chunk = body.arity.to_le_bytes().to_vec();
                    let arity_length = 2;
                    self.assemble_instruction(body.instructions, &mut byte_position, &mut byte_chunk);
                    self.memory.permanent_space.extend_from_slice(&(byte_chunk.len() - arity_length).to_le_bytes());
                    self.memory.permanent_space.extend_from_slice(&byte_chunk)
                },
                _ => unreachable!()
            }
        };
        starts
    }

    fn assemble_instruction(&mut self, instructions: Vec<Instruction>, byte_position: &mut usize, byte_chunk: &mut Vec<u8>) {
        let chunk_size = instructions.len();
        let mut position = 0;
        while position < chunk_size {
            match instructions[position] {
                Instruction::Load(index) => {
                    byte_chunk.push(Bytecode::Load as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    *byte_position += 5
                },
                Instruction::BuiltinCall(index, arity) => {
                    byte_chunk.push(Bytecode::BuiltinCall as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    *byte_position += 5
                },
                Instruction::Call(scope, index) => {
                    byte_chunk.push(Bytecode::Call as u8);
                    byte_chunk.extend_from_slice(&scope.to_le_bytes());
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    *byte_position += 5
                },
                Instruction::RelativeReference(x, y) => {
                    byte_chunk.push(Bytecode::RelativeReference as u8);
                    byte_chunk.extend_from_slice(&x.to_le_bytes());
                    byte_chunk.extend_from_slice(&y.to_le_bytes());
                    *byte_position += 5
                },
                Instruction::Store(index, kind) => {
                    byte_chunk.push(Bytecode::Store as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    byte_chunk.push(kind);
                    *byte_position += 4
                },
                Instruction::LoadVariable(index) => {
                    byte_chunk.push(Bytecode::LoadVariable as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    *byte_position += 3
                },
                Instruction::DefineFunction(index, body_index) => {
                    byte_chunk.push(Bytecode::DefineFunction as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    byte_chunk.extend_from_slice(&body_index.to_le_bytes());
                    *byte_position += 5
                },
                Instruction::Return => {
                    byte_chunk.push(Bytecode::Return as u8);
                    *byte_position += 1
                },
                Instruction::Array(count) => {
                    byte_chunk.push(Bytecode::Array as u8);
                    byte_chunk.extend_from_slice(&count.to_le_bytes());
                    *byte_position += 5
                },
                Instruction::Jump(index) => {
                    let instruction_byte_length = 3;
                    byte_chunk.push(Bytecode::Jump as u8);
                    byte_chunk.extend_from_slice(&((*byte_position + instruction_byte_length + self.get_byte_distance(&instructions, position + 1, index)) as u16).to_le_bytes());
                    *byte_position += 3
                },
                Instruction::JumpIfFalse(index) => {
                    let instruction_byte_length = 3;
                    byte_chunk.push(Bytecode::JumpIfFalse as u8);
                    byte_chunk.extend_from_slice(&((*byte_position + instruction_byte_length + self.get_byte_distance(&instructions, position + 1, index)) as u16).to_le_bytes());
                    *byte_position += 3
                },
            }
            position += 1
        }
    }

    fn get_byte_distance(&self, instructions: &[Instruction], mut position: usize, target: u16) -> usize {
        let mut distance = 0;
        while position < target as usize {
            match instructions[position] {
                Instruction::Return => distance += 1,
                Instruction::LoadVariable(_) | Instruction::Jump(_) | Instruction::JumpIfFalse(_) => distance += 3,
                Instruction::Store(_, _) => distance += 4,
                _ => distance += 5,
            }
            position += 1
        }
        distance
    }
}

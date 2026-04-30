use crate::instructions::{Bytecode, Instruction, Chunk};
use crate::memory::Memory;

pub struct Assembler {
    memory: Memory,
}

impl Assembler {
    pub fn new(memory: Memory) -> Self {
        Self {memory}
    }

    pub fn assemble_map(&mut self, map: Vec<Chunk>) -> Vec<Vec<u8>> {
        let mut byte_map = Vec::new();
        for chunk in map {
            let mut byte_position = 0;
            let mut byte_chunk = Vec::new();
            self.assemble_instruction(chunk.instructions, &mut byte_position, &mut byte_chunk);
            byte_map.push(byte_chunk)
        }
        byte_map
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
                Instruction::Store(index) => {
                    byte_chunk.push(Bytecode::Store as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    *byte_position += 3
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

    fn get_byte_distance(&self, instructions: &Vec<Instruction>, mut position: usize, target: u16) -> usize {
        let mut distance = 0;
        while position < target as usize {
            match instructions[position] {
                Instruction::Return => distance += 1,
                Instruction::Store(_) | Instruction::LoadVariable(_) | Instruction::Jump(_) | Instruction::JumpIfFalse(_) => distance += 3,
                _ => distance += 5,
            }
            position += 1
        }
        distance
    }
}

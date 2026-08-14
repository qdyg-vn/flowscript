use crate::instructions::{Bytecode, Instruction, Chunk};
use crate::memory::Memory;
use crate::constants_pool::ConstantsPool;
use crate::value::HeavyValue;
use crate::virmac::VMConfig;

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

    pub fn assemble_map(mut self, map: Vec<Chunk>) -> (Vec<Vec<u8>>, VMConfig) {
        let mut byte_map = Vec::new();
        for chunk in map {
            let mut byte_position = 0;
            let mut byte_chunk = Vec::new();
            self.assemble_instruction(chunk.instructions, &mut byte_position, &mut byte_chunk);
            byte_map.push(byte_chunk)
        }
        (byte_map, VMConfig {
            heavy_constant_starts: self.assemble_heavy_constants(),
            function_starts: self.assemble_function(),
            memory: self.memory,
            constants_pool: self.constants_pool.constants
        })
    }

    fn assemble_heavy_constants(&mut self) -> Vec<usize> {
        let mut heavy_constant_starts = Vec::new();
        for constant in std::mem::take(&mut self.constants_pool.heavy_constants) {
            match constant {
                HeavyValue::String(string) => {
                    heavy_constant_starts.push(self.memory.permanent_space.len());
                    let string_bytes = string.into_bytes();
                    self.memory.permanent_space.extend_from_slice(&string_bytes.len().to_le_bytes());
                    self.memory.permanent_space.extend_from_slice(&string_bytes)
                },
                _ => unreachable!()
            }
        };
        heavy_constant_starts
    }

    fn assemble_function(&mut self) -> Vec<usize> {
        let mut function_starts = Vec::new();
        for function in std::mem::take(&mut self.constants_pool.functions) {
            function_starts.push(self.memory.functions.len());
            self.memory.functions.extend_from_slice(&function.len().to_le_bytes());
            for chunk in function {
                let mut byte_position = 0;
                let mut byte_chunk = vec![chunk.arity];
                byte_chunk.extend_from_slice(&chunk.variables_count.to_le_bytes());
                let arity_length = 1;
                let variables_count_length = 2;
                self.assemble_instruction(chunk.instructions, &mut byte_position, &mut byte_chunk);
                self.memory.functions.extend_from_slice(&(byte_chunk.len() - arity_length - variables_count_length).to_le_bytes());
                self.memory.functions.extend_from_slice(&byte_chunk)
            }
        };
        function_starts
    }

    fn assemble_instruction(&mut self, instructions: Vec<Instruction>, byte_position: &mut usize, byte_chunk: &mut Vec<u8>) {
        let chunk_size = instructions.len();
        let mut position = 0;
        while position < chunk_size {
            match instructions[position] {
                Instruction::Load(index) => {
                    byte_chunk.push(Bytecode::Load as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    *byte_position += Bytecode::LOAD_SIZE
                },
                Instruction::BuiltinCall(index, arity) => {
                    byte_chunk.push(Bytecode::BuiltinCall as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    *byte_position += Bytecode::BUILTIN_CALL_SIZE
                },
                Instruction::Call(index) => {
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
                Instruction::Store(index, kind) => {
                    byte_chunk.push(Bytecode::Store as u8);
                    byte_chunk.extend_from_slice(&index.to_le_bytes());
                    byte_chunk.push(kind);
                    *byte_position += Bytecode::STORE_SIZE
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
                Instruction::Add(arity, kind) => {
                    byte_chunk.push(Bytecode::Add as u8);
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    byte_chunk.push(kind as u8);
                    *byte_position += Bytecode::ADD_SIZE
                },
                Instruction::Minus(arity, kind) => {
                    byte_chunk.push(Bytecode::Minus as u8);
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    byte_chunk.push(kind as u8);
                    *byte_position += Bytecode::MINUS_SIZE
                },
                Instruction::Multiply(arity, kind) => {
                    byte_chunk.push(Bytecode::Multiply as u8);
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    byte_chunk.push(kind as u8);
                    *byte_position += Bytecode::MULTIPLY_SIZE
                },
                Instruction::Equal(arity, kind) => {
                    byte_chunk.push(Bytecode::Equal as u8);
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    byte_chunk.push(kind as u8);
                    *byte_position += Bytecode::EQUAL_SIZE
                },
                Instruction::LessThan(arity, kind) => {
                    byte_chunk.push(Bytecode::LessThan as u8);
                    byte_chunk.extend_from_slice(&arity.to_le_bytes());
                    byte_chunk.push(kind as u8);
                    *byte_position += Bytecode::LESS_THAN_SIZE
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
                Instruction::Load(_) => distance += Bytecode::LOAD_SIZE,
                Instruction::LoadVariable(_) => distance += Bytecode::LOAD_VARIABLE_SIZE,
                Instruction::Jump(_) => distance += Bytecode::JUMP_SIZE,
                Instruction::JumpIfFalse(_) => distance += Bytecode::JUMP_IF_FALSE_SIZE,
                Instruction::RelativeReference(_, _) => distance += Bytecode::RELATIVE_REFERENCE_SIZE,
                Instruction::Return => distance += Bytecode::RETURN_SIZE,
                Instruction::Store(_, _) => distance += Bytecode::STORE_SIZE,
                Instruction::Array(_) => distance += Bytecode::ARRAY_SIZE,
                Instruction::Not => distance += Bytecode::NOT_SIZE,
                Instruction::Add(_, _) => distance += Bytecode::ADD_SIZE,
                Instruction::Minus(_, _) => distance += Bytecode::MINUS_SIZE,
                Instruction::Multiply(_, _) => distance += Bytecode::MULTIPLY_SIZE,
                Instruction::Equal(_, _) => distance += Bytecode::EQUAL_SIZE,
                Instruction::LessThan(_, _) => distance += Bytecode::LESS_THAN_SIZE,
            }
            position += 1
        }
        distance
    }
}

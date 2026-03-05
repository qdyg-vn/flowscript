use crate::values::Values;

#[derive(Debug)]
pub enum Instruction {
    CALL(u16),
    LOAD(u16),
    STORE(u16),
}

pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Values>,
}
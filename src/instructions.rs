use crate::value::Value;

#[derive(Debug)]
pub enum Instruction {
    Call(u16),
    Load(u16),
    Store(u16),
}

pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
}
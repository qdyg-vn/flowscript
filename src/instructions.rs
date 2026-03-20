use crate::value::Value;

#[derive(Debug)]
pub enum Instruction {
    Call(u16, u16, u16),
    Load(u16),
    Store(u16),
    BuiltinCall(u16, u16),
    GlobalCall(u16, u16)
}

#[derive(Debug)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
}
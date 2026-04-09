#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    Call(u16, u16),
    Load(u16),
    Store(u16, u16),
    LoadVariable(u16, u16),
    BuiltinCall(u16, u16),
    RelativeReference(u16, u16),
    DefineFunction(u16, u16),
    Jump(u16),
    JumpIfFalse(u16),
    Return,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub arity: u16,
}

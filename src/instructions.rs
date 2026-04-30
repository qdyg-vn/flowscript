#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bytecode {
    Call, // u16: scope, u16: offset from base pointer
    BuiltinCall, // u16: index in builtin table, u16: arity
    Load, // u32: index in constant pool
    HeavyLoad, // u32: index in constant pool
    LoadVariable, // u16: offset from base pointer
    Jump, // u16: instruction position
    JumpIfFalse, // u16: instruction position
    RelativeReference, // u16: x, u16: y
    Return,
    Store, // u16: offset from base pointer
    Array, // u32: length
    DefineFunction, // u16: offset from base pointer, u16: index of body in constant pool
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    Call(u16, u16),
    BuiltinCall(u16, u16),
    Load(u32),
    LoadVariable(u16),
    Jump(u16),
    JumpIfFalse(u16),
    RelativeReference(u16, u16),
    Return,
    Store(u16),
    Array(u32),
    DefineFunction(u16, u16),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub arity: u16,
}

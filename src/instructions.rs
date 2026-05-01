#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bytecode {
    Call, // u16: scope, u16: offset from base pointer
    BuiltinCall, // u16: index in builtin table, u16: arity
    Load, // u32: index in constant pool
    LoadVariable, // u16: offset from base pointer
    Jump, // u16: instruction position
    JumpIfFalse, // u16: instruction position
    RelativeReference, // u16: x, u16: y
    Return,
    Store, // u16: offset from base pointer
    Array, // u32: length
    DefineFunction, // u16: offset from base pointer, u16: index of body in constant pool
}

impl Bytecode {
    pub const CALL: u8 = Bytecode::Call as u8;
    pub const BUILTINCALL: u8 = Bytecode::BuiltinCall as u8;
    pub const LOAD: u8 = Bytecode::Load as u8;
    pub const LOADVARIABLE: u8 = Bytecode::LoadVariable as u8;
    pub const JUMP: u8 = Bytecode::Jump as u8;
    pub const JUMPIFFALSE: u8 = Bytecode::JumpIfFalse as u8;
    pub const RELATIVEREFERENCE: u8 = Bytecode::RelativeReference as u8;
    pub const RETURN: u8 = Bytecode::Return as u8;
    pub const STORE: u8 = Bytecode::Store as u8;
    pub const ARRAY: u8 = Bytecode::Array as u8;
    pub const DEFINEFUNCTION: u8 = Bytecode::DefineFunction as u8;
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

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
    Store, // u16: offset from base pointer, u8: kind
    Array, // u32: length
    DefineFunction, // u16: offset from base pointer, u16: index of body in constant pool
}

impl Bytecode {
    pub const CALL: u8 = Bytecode::Call as u8;
    pub const CALL_SIZE: usize = 1 + 2 + 2;
    pub const BUILTIN_CALL: u8 = Bytecode::BuiltinCall as u8;
    pub const BUILTIN_CALL_SIZE: usize = 1 + 2 + 2;
    pub const LOAD: u8 = Bytecode::Load as u8;
    pub const LOAD_SIZE: usize = 1 + 4;
    pub const LOAD_VARIABLE: u8 = Bytecode::LoadVariable as u8;
    pub const LOAD_VARIABLE_SIZE: usize = 1 + 2;
    pub const JUMP: u8 = Bytecode::Jump as u8;
    pub const JUMP_SIZE: usize = 1 + 2;
    pub const JUMP_IF_FALSE: u8 = Bytecode::JumpIfFalse as u8;
    pub const JUMP_IF_FALSE_SIZE: usize = 1 + 2;
    pub const RELATIVE_REFERENCE: u8 = Bytecode::RelativeReference as u8;
    pub const RELATIVE_REFERENCE_SIZE: usize = 1 + 2 + 2;
    pub const RETURN: u8 = Bytecode::Return as u8;
    pub const RETURN_SIZE: usize = 1;
    pub const STORE: u8 = Bytecode::Store as u8;
    pub const STORE_SIZE: usize = 1 + 2 + 1;
    pub const ARRAY: u8 = Bytecode::Array as u8;
    pub const ARRAY_SIZE: usize = 1 + 4;
    pub const DEFINE_FUNCTION: u8 = Bytecode::DefineFunction as u8;
    pub const DEFINE_FUNCTION_SIZE: usize = 1 + 2 + 2;
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
    Store(u16, u8),
    Array(u32),
    DefineFunction(u16, u16),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub arity: u16,
}

use crate::value::Kind;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Not,
    Add, // u16: arity, u8: kind
    Minus, // u16: arity, u8: kind
    Multiply, // u16: arity, u8: kind
    Equal, // u16: arity, u8: kind
    LessThan, // u16: arity, u8: kind
}

impl Bytecode {
    pub const CALL: u8 = Self::Call as u8;
    pub const CALL_SIZE: usize = 1 + 2;
    pub const BUILTIN_CALL: u8 = Self::BuiltinCall as u8;
    pub const BUILTIN_CALL_SIZE: usize = 1 + 2 + 2;
    pub const LOAD: u8 = Self::Load as u8;
    pub const LOAD_SIZE: usize = 1 + 4;
    pub const LOAD_VARIABLE: u8 = Self::LoadVariable as u8;
    pub const LOAD_VARIABLE_SIZE: usize = 1 + 2;
    pub const JUMP: u8 = Self::Jump as u8;
    pub const JUMP_SIZE: usize = 1 + 2;
    pub const JUMP_IF_FALSE: u8 = Self::JumpIfFalse as u8;
    pub const JUMP_IF_FALSE_SIZE: usize = 1 + 2;
    pub const RELATIVE_REFERENCE: u8 = Self::RelativeReference as u8;
    pub const RELATIVE_REFERENCE_SIZE: usize = 1 + 2 + 2;
    pub const RETURN: u8 = Self::Return as u8;
    pub const RETURN_SIZE: usize = 1;
    pub const STORE: u8 = Self::Store as u8;
    pub const STORE_SIZE: usize = 1 + 2 + 1;
    pub const ARRAY: u8 = Self::Array as u8;
    pub const ARRAY_SIZE: usize = 1 + 4;
    pub const NOT: u8 = Self::Not as u8;
    pub const NOT_SIZE: usize = 1;
    pub const ADD: u8 = Self::Add as u8;
    pub const ADD_SIZE: usize = 1 + 2 + 1;
    pub const MINUS: u8 = Self::Minus as u8;
    pub const MINUS_SIZE: usize = 1 + 2 + 1;
    pub const MULTIPLY: u8 = Self::Multiply as u8;
    pub const MULTIPLY_SIZE: usize = 1 + 2 + 1;
    pub const EQUAL: u8 = Self::Equal as u8;
    pub const EQUAL_SIZE: usize = 1 + 2 + 1;
    pub const LESS_THAN: u8 = Self::LessThan as u8;
    pub const LESS_THAN_SIZE: usize = 1 + 2 + 1;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Instruction {
    Call(u16),
    BuiltinCall(u16, u16),
    Load(u32),
    LoadVariable(u16),
    Jump(u16),
    JumpIfFalse(u16),
    RelativeReference(u16, u16),
    Return,
    Store(u16, u8),
    Array(u32),
    Not,
    Add(u16, Kind),
    Minus(u16, Kind),
    Multiply(u16, Kind),
    Equal(u16, Kind),
    LessThan(u16, Kind),
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub arity: u8,
    pub variables_count: u16,
}

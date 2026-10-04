use crate::value::{Kind, LightValue};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bytecode {
    Call, // u32: offset from base pointer
    BuiltinCall, // u16: index in builtin table, u16: arity
    Load, // u8: kind, bool/f64/i64/u32: value
    LoadVariable, // u16: offset from base pointer
    Jump, // u16: instruction position
    JumpIfFalse, // u16: instruction position
    RelativeReference, // u16: x, u16: y
    Return,
    Store, // u16: offset from base pointer
    StoreTemp, // u8: temp index
    StationCapture, // u16: offset from reference pointer
    Array, // u32: length
    Not,
    Reverse,
    Add, // u16: arity, u8: kind
    Minus, // u16: arity, u8: kind
    Multiply, // u16: arity, u8: kind
    Equal, // u16: arity, u8: kind
    LessThan, // u16: arity, u8: kind
    StructInstantiation, // u8: fields count
    FieldAccess, // u16: offset from base pointer, u16: field index
}

impl Bytecode {
    pub const CALL: u8 = Self::Call as u8;
    pub const CALL_SIZE: usize = 1 + 4;
    pub const BUILTIN_CALL: u8 = Self::BuiltinCall as u8;
    pub const BUILTIN_CALL_SIZE: usize = 1 + 2 + 2;
    pub const LOAD: u8 = Self::Load as u8;
    pub const LOAD_SIZE: usize = 1 + 1;
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
    pub const STORE_SIZE: usize = 1 + 2;
    pub const STORE_TEMP: u8 = Self::StoreTemp as u8;
    pub const STORE_TEMP_SIZE: usize = 1 + 1;
    pub const STATION_CAPTURE: u8 = Self::StationCapture as u8;
    pub const STATION_CAPTURE_SIZE: usize = 1 + 2;
    pub const ARRAY: u8 = Self::Array as u8;
    pub const ARRAY_SIZE: usize = 1 + 4;
    pub const NOT: u8 = Self::Not as u8;
    pub const NOT_SIZE: usize = 1;
    pub const REVERSE: u8 = Self::Reverse as u8;
    pub const REVERSE_SIZE: usize = 1 + 2;
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
    pub const STRUCT_INSTANTIATION: u8 = Self::StructInstantiation as u8;
    pub const STRUCT_INSTANTIATION_SIZE: usize = 1 + 1;
    pub const FIELD_ACCESS: u8 = Self::FieldAccess as u8;
    pub const FIELD_ACCESS_SIZE: usize = 1 + 2 + 2;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Instruction {
    Call(u32),
    BuiltinCall(u16, u16),
    Load(LightValue),
    HeavyLoad(u32, Kind),
    LoadVariable(u16),
    Jump(u16),
    JumpIfFalse(u16),
    RelativeReference(u16, u16),
    Return,
    Store(u16),
    StationCapture(u16),
    StoreTemp(u8),
    Array(u32),
    Not,
    Reverse(u16),
    Add(u16, Kind),
    Minus(u16, Kind),
    Multiply(u16, Kind),
    Equal(u16, Kind),
    LessThan(u16, Kind),
    StructInstantiation(u8),
    FieldAccess(u16, u16),
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub variables_count: u16,
    pub arity: u8,
    pub max_temp_variables: u8,
    pub max_relative_reference: u8,
}

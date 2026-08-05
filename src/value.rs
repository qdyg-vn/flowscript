use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use crate::instructions::Chunk;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Kind {
    Boolean,
    Float,
    Integer,
    String,
    Array,
    Nil,
    Undefined,
}

impl Kind {
    pub const BOOLEAN: u8 = Self::Boolean as u8;
    pub const FLOAT: u8 = Self::Float as u8;
    pub const INTEGER: u8 = Self::Integer as u8;
    pub const STRING: u8 = Self::String as u8;
    pub const ARRAY: u8 = Self::Array as u8;
}


pub fn get_kind(kind: u8) -> Kind {
    match kind {
        Kind::BOOLEAN => Kind::Boolean,
        Kind::FLOAT => Kind::Float,
        Kind::INTEGER => Kind::Integer,
        Kind::STRING => Kind::String,
        Kind::ARRAY => Kind::Array,
        _ => unreachable!()
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean => write!(f, "boolean"),
            Self::Float => write!(f, "float"),
            Self::Integer => write!(f, "integer"),
            Self::String => write!(f, "string"),
            Self::Array => write!(f, "array"),
            Self::Nil => write!(f, "nil"),
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightValue {
    Boolean(bool),
    Nil,
    Float(f64),
    Integer(i64),
    StringPointer(u32),
    FunctionPointer(u32),
    ClosurePointer(u32),
    ArrayPointer(u32),
    StringHeapPointer(u32),
    ArrayHeapPointer(u32),
}

impl LightValue {
    pub const BOOLEAN: u8 = 0;
    pub const BOOLEAN_SIZE: usize = 1 + 1;
    pub const NIL: u8 = 1;
    pub const NIL_SIZE: usize = 1;
    pub const FLOAT: u8 = 2;
    pub const FLOAT_SIZE: usize = 1 + 8;
    pub const INTEGER: u8 = 3;
    pub const INTEGER_SIZE: usize = 1 + 8;  
    pub const STRING_POINTER: u8 = 4;
    pub const STRING_POINTER_SIZE: usize = 1 + 4;
    pub const FUNCTION_POINTER: u8 = 5;
    pub const FUNCTION_POINTER_SIZE: usize = 1 + 4;
    pub const ARRAY_POINTER: u8 = 6;
    pub const ARRAY_POINTER_SIZE: usize = 1 + 4;
    pub fn get_kind(&self) -> Kind {
        match self {
            Self::Boolean(_) => Kind::Boolean,
            Self::Integer(_) => Kind::Integer,
            Self::Float(_) => Kind::Float,
            _ => todo!()
        }
    }
}

impl Eq for LightValue {}

impl Hash for LightValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Boolean(boolean) => boolean.hash(state),
            Self::Integer(integer) => integer.hash(state),
            Self::Float(float) => float.to_bits().hash(state),
            Self::StringPointer(string) => string.hash(state),
            Self::Nil => 0.hash(state),
            Self::FunctionPointer(function) => function.hash(state),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeavyValue {
    String(String),
    Function(Vec<Chunk>),
    Closure(Closure),
    Array(Vec<LightValue>),
}

impl HeavyValue {
    pub fn get_kind(&self) -> Kind {
        match self {
            Self::String(_) => Kind::String,
            _ => todo!()
        }
    }
}

impl Eq for HeavyValue {}

impl Hash for HeavyValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::String(string) => string.hash(state),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Nil,
    Float(f64),
    Integer(i64),
    String(String),
    Function(Vec<u8>),
    Array(Vec<Value>),
}

impl Value {
    pub fn get_kind(&self) -> Kind {
        match self {
            Self::Boolean(_) => Kind::Boolean,
            Self::Float(_) => Kind::Float,
            Self::Integer(_) => Kind::Integer,
            Self::String(_) => Kind::String,
            _ => todo!()
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(boolean) => write!(f, "{}", boolean),
            Self::Nil => write!(f, "Nil"),
            Self::Float(float) => write!(f, "{}", float),
            Self::Integer(integer) => write!(f, "{}", integer),
            Self::String(string) => write!(f, "{}", string),
            Self::Array(elements) => {
                write!(f, "[")?;
                for (index, element) in elements.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, "]")
            },
            something => write!(f, "{:?}", something)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Upvalue {
    Open(usize),
    Closed(LightValue)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Closure {
    pub function: Rc<LightValue>,
    pub upvalue: Vec<Rc<RefCell<Upvalue>>>,
}

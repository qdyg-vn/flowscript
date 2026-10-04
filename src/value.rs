use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub const BOOLEAN_SIZE: usize = 1;
pub const FLOAT_SIZE: usize = 8;
pub const INTEGER_SIZE: usize = 8;
pub const INDEX_SIZE: usize = 4;
pub const NIL_SIZE: usize = 0;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Tag {
    Nil,
    Boolean,
    Integer,
    Float,
    String,
    StringPointer,
    StringHeapPointer,
    Array,
    ArrayPointer,
    Struct,
    StructPointer,
}

impl Tag {
    pub const NIL: u8 = Self::Nil as u8;
    pub const BOOLEAN: u8 = Self::Boolean as u8;
    pub const INTEGER: u8 = Self::Integer as u8;
    pub const FLOAT: u8 = Self::Float as u8;
    pub const STRING: u8 = Self::String as u8;
    pub const STRING_POINTER: u8 = Self::StringPointer as u8;
    pub const STRING_HEAP_POINTER: u8 = Self::StringHeapPointer as u8;
    pub const ARRAY: u8 = Self::Array as u8;
    pub const ARRAY_POINTER: u8 = Self::ArrayPointer as u8;
    pub const STRUCT: u8 = Self::Struct as u8;
    pub const STRUCT_POINTER: u8 = Self::StructPointer as u8;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Kind {
    Boolean,
    Float,
    Integer,
    String,
    Array,
    Struct(u16),
    Nil,
    Undefined,
}

impl Kind {
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::Boolean => Tag::BOOLEAN,
            Self::Float => Tag::FLOAT,
            Self::Integer => Tag::INTEGER,
            Self::String => Tag::STRING,
            Self::Array => Tag::ARRAY,
            Self::Struct(_) => Tag::STRUCT,
            Self::Nil => Tag::NIL,
            Self::Undefined => unreachable!(),
        }
    }

    pub fn size(&self) -> u16 {
        match self {
            Self::Boolean => LightValue::BOOLEAN_SIZE as u16,
            Self::Float => LightValue::FLOAT_SIZE as u16,
            Self::Integer => LightValue::INTEGER_SIZE as u16,
            Self::String => LightValue::STRING_POINTER_SIZE as u16,
            Self::Array => LightValue::ARRAY_POINTER_SIZE as u16,
            Self::Struct(_) => LightValue::STRUCT_POINTER_SIZE as u16,
            _ => unreachable!(),
        }
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
    StringHeapPointer(u32),
    ArrayPointer(u32),
    StructPointer(u32),
}

impl LightValue {
    pub const BOOLEAN_SIZE: usize = 1 + 1;
    pub const NIL_SIZE: usize = 1;
    pub const FLOAT_SIZE: usize = 1 + 8;
    pub const INTEGER_SIZE: usize = 1 + 8;
    pub const STRING_POINTER_SIZE: usize = 1 + 4;
    pub const ARRAY_POINTER_SIZE: usize = 1 + 4;
    pub const STRUCT_POINTER_SIZE: usize = 1 + 4;
    pub fn get_kind(&self) -> Kind {
        match self {
            Self::Boolean(_) => Kind::Boolean,
            Self::Integer(_) => Kind::Integer,
            Self::Float(_) => Kind::Float,
            _ => unreachable!(),
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
    Array(Vec<Value>),
    Struct(Vec<Value>),
}

impl Value {
    pub fn get_kind(&self) -> Kind {
        match self {
            Self::Boolean(_) => Kind::Boolean,
            Self::Float(_) => Kind::Float,
            Self::Integer(_) => Kind::Integer,
            Self::String(_) => Kind::String,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(boolean) => write!(f, "{}", boolean),
            Self::Nil => write!(f, ""),
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
            _ => unreachable!()
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::String(string) => string.hash(state),
            _ => unreachable!(),
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

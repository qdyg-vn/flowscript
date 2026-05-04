use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use crate::instructions::Chunk;

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
}

impl LightValue {
    pub const BOOLEAN: u8 = 0;
    pub const NIL: u8 = 1;
    pub const FLOAT: u8 = 2;
    pub const INTEGER: u8 = 3;
    pub const STRINGPOINTER: u8 = 4;
    pub const FUNCTIONPOINTER: u8 = 5;
    pub const ARRAYPOINTER: u8 = 6;
    pub fn to_byte(&self) -> Vec<u8> {
        match self {
            LightValue::Boolean(boolean) => vec![*boolean as u8],
            LightValue::Nil => vec![0],
            LightValue::Float(float) => float.to_le_bytes().to_vec(),
            LightValue::Integer(integer) => integer.to_le_bytes().to_vec(),
            LightValue::StringPointer(index) | LightValue::FunctionPointer(index) | LightValue::ArrayPointer(index) => index.to_le_bytes().to_vec(),
            _ => unreachable!()
        }
    }
}

impl Eq for LightValue {}

impl Hash for LightValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            LightValue::Boolean(boolean) => boolean.hash(state),
            LightValue::Integer(integer) => integer.hash(state),
            LightValue::Float(float) => float.to_bits().hash(state),
            LightValue::StringPointer(string) => string.hash(state),
            LightValue::Nil => 0.hash(state),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeavyValue {
    String(String),
    Function(Chunk),
    Closure(Closure),
    Array(Vec<LightValue>),
}

impl Eq for HeavyValue {}

impl Hash for HeavyValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            HeavyValue::String(string) => string.hash(state),
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Nil => write!(f, "Nil"),
            Value::Float(float) => write!(f, "{}", float),
            Value::Integer(integer) => write!(f, "{}", integer),
            Value::Array(elements) => {
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

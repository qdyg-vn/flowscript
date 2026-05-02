use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use crate::instructions::Chunk;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Nil,
    Float(f64),
    Integer(i64),
    StringPointer(u32),
    FunctionPointer(u32),
    ClosurePointer(u32),
    ArrayPointer(u32),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Nil => write!(f, "Nil"),
            Value::Float(float) => write!(f, "{}", float),
            Value::Integer(integer) => write!(f, "{}", integer),
            something => write!(f, "{:?}", something)
        }
    }
}

impl Value {
    pub fn to_byte(&self) -> Vec<u8> {
        match self {
            Value::Boolean(boolean) => vec![*boolean as u8],
            Value::Nil => vec![0],
            Value::Float(float) => float.to_le_bytes().to_vec(),
            Value::Integer(integer) => integer.to_le_bytes().to_vec(),
            Value::StringPointer(index) | Value::FunctionPointer(index) | Value::ArrayPointer(index) => index.to_le_bytes().to_vec(),
            _ => unreachable!()
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Boolean(boolean) => boolean.hash(state),
            Value::Integer(integer) => integer.hash(state),
            Value::Float(float) => float.to_bits().hash(state),
            Value::StringPointer(string) => string.hash(state),
            Value::Nil => 0.hash(state),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeavyValue {
    String(String),
    Function(Chunk),
    Closure(Closure),
    Array(Vec<Value>),
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Upvalue {
    Open(usize),
    Closed(Value)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Closure {
    pub function: Rc<Value>,
    pub upvalue: Vec<Rc<RefCell<Upvalue>>>,
}

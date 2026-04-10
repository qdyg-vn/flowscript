use crate::instructions::Chunk;
use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Nil,
    Float(f64),
    Integer(i64),
    String(Rc<String>),
    Function(Rc<Chunk>),
    Closure(Closure),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Nil => write!(f, "Nil"),
            Value::Float(float) => write!(f, "{}", float),
            Value::String(string) => write!(f, "{}", string),
            Value::Integer(integer) => write!(f, "{}", integer),
            something => write!(f, "{:?}", something)
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
            Value::String(string) => string.hash(state),
            Value::Nil => 0.hash(state),
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

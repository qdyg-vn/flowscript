use std::fmt;
use std::hash::{Hash, Hasher};
use crate::instructions::Chunk;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Nil,
    Float(f64),
    String(String),
    Integer(i64),
    Function(Box<Chunk>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Nil => write!(f, "Nil"),
            Value::Float(float) => write!(f, "{}", float),
            Value::String(string) => write!(f, "{}", string),
            Value::Integer(integer) => write!(f, "{}", integer),
            _ => todo!()
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

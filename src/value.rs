use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use crate::instructions::Chunk;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Boolean,
    Float,
    Integer,
    String,
    Array,
}

impl Kind {
    pub const BOOLEAN: u8 = Kind::Boolean as u8;
    pub const FLOAT: u8 = Kind::Float as u8;
    pub const INTEGER: u8 = Kind::Integer as u8;
    pub const STRING: u8 = Kind::String as u8;
    pub const ARRAY: u8 = Kind::Array as u8;
    pub fn get_variable_type(&self) -> VariableType {
        match self {
            Kind::Boolean => VariableType::Boolean,
            Kind::Float => VariableType::Float,
            Kind::Integer => VariableType::Integer,
            Kind::String => VariableType::String,
            _ => todo!()
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Boolean => write!(f, "boolean"),
            Kind::Float => write!(f, "float"),
            Kind::Integer => write!(f, "integer"),
            Kind::String => write!(f, "string"),
            Kind::Array => write!(f, "array"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VariableType {
    Boolean,
    Float,
    Integer,
    String,
    Function(u32),
    Dynamic,
}

impl VariableType {
    pub fn get_kind(&self) -> Kind {
        match self {
            VariableType::Boolean => Kind::Boolean,
            VariableType::Float => Kind::Float,
            VariableType::Integer => Kind::Integer,
            VariableType::String => Kind::String,
            _ => todo!()
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
    pub const STRINGPOINTER: u8 = 4;
    pub const STRINGPOINTER_SIZE: usize = 1 + 4;
    pub const FUNCTIONPOINTER: u8 = 5;
    pub const FUNCTIONPOINTER_SIZE: usize = 1 + 4;
    pub const ARRAYPOINTER: u8 = 6;
    pub const ARRAYPOINTER_SIZE: usize = 1 + 4;
    pub fn get_kind(&self) -> Kind {
        match self {
            LightValue::Boolean(_) => Kind::Boolean,
            LightValue::Integer(_) => Kind::Integer,
            LightValue::Float(_) => Kind::Float,
            _ => todo!()
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
            LightValue::FunctionPointer(function) => function.hash(state),
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

impl HeavyValue {
    pub fn get_kind(&self) -> Kind {
        match self {
            HeavyValue::String(_) => Kind::String,
            _ => todo!()
        }
    }
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

impl Value {
    pub fn get_kind(&self) -> Kind {
        match self {
            Value::Boolean(_) => Kind::Boolean,
            Value::Float(_) => Kind::Float,
            Value::Integer(_) => Kind::Integer,
            Value::String(_) => Kind::String,
            _ => todo!()
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Nil => write!(f, "Nil"),
            Value::Float(float) => write!(f, "{}", float),
            Value::Integer(integer) => write!(f, "{}", integer),
            Value::String(string) => write!(f, "{}", string),
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

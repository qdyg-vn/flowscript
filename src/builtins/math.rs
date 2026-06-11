use crate::value::Value;
use crate::error_handler::{Error, RuntimeError, RuntimeErrorType, TypeError, TypeErrorType};

pub fn add(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.is_empty() { return Err(Error::RuntimeError(RuntimeError { kind: RuntimeErrorType::InsufficientOperands("Addition".to_string()) })) }
    let (first, rest) = arguments.split_first().unwrap();
    if rest.is_empty() { return match first {
        Value::Integer(_) | Value::Float(_) => Ok(first.to_owned()),
        _ => Err(Error::TypeError(TypeError { kind: TypeErrorType::InvalidUnaryOperand("Unary add only supports numeric types".to_string()) }))
    } };
    rest.iter().try_fold(first.clone(), |accumulator, x| {
        match (&accumulator, x) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a.to_string() + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            _ => Err(Error::TypeError(TypeError { kind: TypeErrorType::TypeMismatch(accumulator, x.to_owned()) }))
        }
    })
}

pub fn minus(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.is_empty() { return Err(Error::RuntimeError(RuntimeError { kind: RuntimeErrorType::InsufficientOperands("Minus".to_string()) })) }
    let (first, rest) = arguments.split_first().unwrap();
    if rest.is_empty() {
        return match first {
            Value::Integer(a) => Ok(Value::Integer(-a)),
            Value::Float(a) => Ok(Value::Float(-a)),
            _ => Err(Error::TypeError(TypeError { kind: TypeErrorType::InvalidUnaryOperand("Unary minus only supports numeric types".to_string()) }))
        }
    };
    rest.iter().try_fold(first.clone(), |accumulator, x| {
        match (&accumulator, x) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a.replacen(&**b, "", 1))),
            _ => Err(Error::TypeError(TypeError { kind: TypeErrorType::TypeMismatch(accumulator, x.to_owned()) }))
        }
    })
}

pub fn multiply(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.is_empty() { return Err(Error::RuntimeError(RuntimeError { kind: RuntimeErrorType::InsufficientOperands("Multiply".to_string()) })) }
    let (first, rest) = arguments.split_first().unwrap();
    if rest.is_empty() {
        return Err(Error::TypeError(TypeError { kind: TypeErrorType::InvalidOperand("Multiplication is a binary operator".to_string()) }))
    };
    rest.iter().try_fold(first.clone(), |accumulator, x| {
        match (&accumulator, x) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::String(a), Value::Integer(b)) => Ok(Value::String(a.repeat(*b as usize))),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            _ => Err(Error::TypeError(TypeError { kind: TypeErrorType::TypeMismatch(accumulator, x.to_owned()) }))
        }
    })
}

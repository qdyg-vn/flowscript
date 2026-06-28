use crate::error_handler::{Error, RuntimeError, RuntimeErrorType, TypeError, TypeErrorType};
use crate::value::Value;

pub fn equal(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.is_empty() { return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("==".to_string())}.into()) }
    let result = match (arguments[0].clone(), arguments[1].clone()) {
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (a, b) => return Err(TypeError {kind: TypeErrorType::TypeMismatch(a.get_kind(), b.get_kind())}.into())
    };
    Ok(Value::Boolean(result))
}

pub fn less(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.is_empty() { return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("<".to_string())}.into()) }
    let result = match (arguments[0].clone(), arguments[1].clone()) {
        (Value::Float(a), Value::Float(b)) => a < b,
        (Value::Integer(a), Value::Integer(b)) => a < b,
        (Value::Boolean(a), Value::Boolean(b)) => a < b,
        (Value::String(a), Value::String(b)) => a < b,
        (a, b) => return Err(TypeError {kind: TypeErrorType::TypeMismatch(a.get_kind(), b.get_kind())}.into())
    };
    Ok(Value::Boolean(result))
}

pub fn greater(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.is_empty() { return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands(">".to_string())}.into()) }
    let result = match (arguments[0].clone(), arguments[1].clone()) {
        (Value::Float(a), Value::Float(b)) => a > b,
        (Value::Integer(a), Value::Integer(b)) => a > b,
        (Value::Boolean(a), Value::Boolean(b)) => a > b,
        (Value::String(a), Value::String(b)) => a > b,
        (a, b) => return Err(TypeError {kind: TypeErrorType::TypeMismatch(a.get_kind(), b.get_kind())}.into())
    };
    Ok(Value::Boolean(result))
}

pub fn not_equal(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.is_empty() { return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("!=".to_string())}.into()) }
    let result = match (arguments[0].clone(), arguments[1].clone()) {
        (Value::Float(a), Value::Float(b)) => a != b,
        (Value::Integer(a), Value::Integer(b)) => a != b,
        (Value::Boolean(a), Value::Boolean(b)) => a != b,
        (Value::String(a), Value::String(b)) => a != b,
        (a, b) => return Err(TypeError {kind: TypeErrorType::TypeMismatch(a.get_kind(), b.get_kind())}.into())
    };
    Ok(Value::Boolean(result))
}

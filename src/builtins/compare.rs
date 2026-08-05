use crate::error_handler::{Error, RuntimeError, RuntimeErrorType, TypeError, TypeErrorType};
use crate::value::Value;

pub fn equal(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.len() < 2 { return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("==".to_string())}.into()) }
    for window in arguments.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let is_equal = match (a, b) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            _ => return Err(TypeError {kind: TypeErrorType::TypeMismatch(a.get_kind(), b.get_kind())}.into())
        };
        if !is_equal {
            return Ok(Value::Boolean(false))
        }
    }
    Ok(Value::Boolean(true))
}

pub fn lower_than(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.len() < 2 { return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("<".to_string())}.into()) }
    for window in arguments.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let is_lower_than = match (a, b) {
            (Value::String(a), Value::String(b)) => a < b,
            (Value::Float(a), Value::Float(b)) => a < b,
            (Value::Integer(a), Value::Integer(b)) => a < b,
            _ => return Err(TypeError {kind: TypeErrorType::TypeMismatch(a.get_kind(), b.get_kind())}.into())
        };
        if !is_lower_than {
            return Ok(Value::Boolean(false))
        }
    }
    Ok(Value::Boolean(true))
}

pub fn greater_than(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.len() < 2 { return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands(">".to_string())}.into()) }
    for window in arguments.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let is_greater_than = match (a, b) {
            (Value::String(a), Value::String(b)) => a > b,
            (Value::Float(a), Value::Float(b)) => a > b,
            (Value::Integer(a), Value::Integer(b)) => a > b,
            _ => return Err(TypeError {kind: TypeErrorType::TypeMismatch(a.get_kind(), b.get_kind())}.into())
        };
        if !is_greater_than {
            return Ok(Value::Boolean(false))
        }
    }
    Ok(Value::Boolean(true))
}

pub fn lower_than_or_equal(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.len() < 2 { return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("<=".to_string())}.into()) }
    for window in arguments.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let is_lower_than = match (a, b) {
            (Value::String(a), Value::String(b)) => a <= b,
            (Value::Float(a), Value::Float(b)) => a <= b,
            (Value::Integer(a), Value::Integer(b)) => a <= b,
            _ => return Err(TypeError {kind: TypeErrorType::TypeMismatch(a.get_kind(), b.get_kind())}.into())
        };
        if !is_lower_than {
            return Ok(Value::Boolean(false))
        }
    }
    Ok(Value::Boolean(true))
}

pub fn greater_than_or_equal(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.len() < 2 { return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands(">=".to_string())}.into()) }
    for window in arguments.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let is_greater_than_or_equal = match (a, b) {
            (Value::String(a), Value::String(b)) => a >= b,
            (Value::Float(a), Value::Float(b)) => a >= b,
            (Value::Integer(a), Value::Integer(b)) => a >= b,
            _ => return Err(TypeError {kind: TypeErrorType::TypeMismatch(a.get_kind(), b.get_kind())}.into())
        };
        if !is_greater_than_or_equal {
            return Ok(Value::Boolean(false))
        }
    }
    Ok(Value::Boolean(true))
}

pub fn not_equal(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.len() < 2 { return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("!=".to_string())}.into()) }
    for window in arguments.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let is_not_equal = match (a, b) {
            (Value::String(a), Value::String(b)) => a != b,
            (Value::Boolean(a), Value::Boolean(b)) => a != b,
            (Value::Integer(a), Value::Integer(b)) => a != b,
            (Value::Float(a), Value::Float(b)) => a != b,
            _ => return Err(TypeError {kind: TypeErrorType::TypeMismatch(a.get_kind(), b.get_kind())}.into())
        };
        if !is_not_equal {
            return Ok(Value::Boolean(false))
        }
    }
    Ok(Value::Boolean(true))
}

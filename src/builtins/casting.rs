use crate::error_handler::{Error, RuntimeError, RuntimeErrorType};
use crate::value::Value;

pub fn to_string(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.is_empty() {
        return Err(Error::RuntimeError(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("String".to_string())}))
    }
    let result = match &arguments[0] {
        Value::Float(float) => float.to_string(),
        Value::Integer(integer) => integer.to_string(),
        Value::Boolean(boolean) => boolean.to_string(),
        Value::Nil => "Nil".to_string(),
        Value::String(_) => return Ok(arguments[0].clone()),
        _ => todo!(),
    };
    Ok(Value::String(result))
}

pub fn to_integer(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.is_empty() {
        return Err(Error::RuntimeError(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("Integer".to_string())}))
    }
    let result = match &arguments[0] {
        Value::Float(float) => *float as i64,
        Value::Integer(integer) => *integer,
        Value::Boolean(boolean) => *boolean as i64,
        Value::Nil => 0,
        Value::String(string) => match string.parse() {
            Ok(integer) => integer,
            Err(_) => return Err(Error::RuntimeError(RuntimeError {kind: RuntimeErrorType::ParseError(string.clone(), "Integer".to_string())}))
        },
        _ => todo!(),
    };
    Ok(Value::Integer(result))
}

pub fn to_float(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.is_empty() {
        return Err(Error::RuntimeError(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("Float".to_string())}))
    }
    let result = match &arguments[0] {
        Value::Float(float) => *float,
        Value::Integer(integer) => *integer as f64,
        Value::Boolean(_) => return Err(Error::RuntimeError(RuntimeError {kind: RuntimeErrorType::ParseError("Boolean".to_string(), "Float".to_string())})),
        Value::Nil => return Err(Error::RuntimeError(RuntimeError {kind: RuntimeErrorType::ParseError("Nil".to_string(), "Float".to_string())})),
        Value::String(string) => match string.parse() {
            Ok(float) => float,
            Err(_) => return Err(Error::RuntimeError(RuntimeError {kind: RuntimeErrorType::ParseError(string.clone(), "Float".to_string())}))
        },
        _ => todo!(),
    };
    Ok(Value::Float(result))
}

pub fn to_boolean(arguments: &[Value]) -> Result<Value, Error> {
    if arguments.is_empty() {
        return Err(Error::RuntimeError(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("Boolean".to_string())}))
    }
    let result = match &arguments[0] {
        Value::Float(float) => *float != 0.0,
        Value::Integer(integer) => *integer != 0,
        Value::Boolean(boolean) => *boolean,
        Value::Nil => false,
        Value::String(_) => true,
        _ => todo!(),
    };
    Ok(Value::Boolean(result))
}

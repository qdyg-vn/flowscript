use crate::error_handler::{Error, RuntimeError, RuntimeErrorType};
use crate::value::Value;

pub fn to_string(mut arguments: Vec<Value>) -> Result<Value, Error> {
    if arguments.is_empty() {
        return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("String".to_string())}.into())
    } else if arguments.len() > 1 {
        todo!()
    }
    let value = arguments.pop().unwrap();
    let result = match &value {
        Value::Float(float) => float.to_string(),
        Value::Integer(integer) => integer.to_string(),
        Value::Boolean(boolean) => boolean.to_string(),
        Value::Nil => String::new(),
        Value::String(_) => return Ok(value),
        _ => todo!(),
    };
    Ok(Value::String(result))
}

pub fn to_integer(mut arguments: Vec<Value>) -> Result<Value, Error> {
    if arguments.is_empty() {
        return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("Integer".to_string())}.into())
    } else if arguments.len() > 1 {
        todo!()
    }
    let value = arguments.pop().unwrap();
    let result = match value {
        Value::Float(float) => float as i64,
        Value::Integer(integer) => integer,
        Value::Boolean(boolean) => boolean as i64,
        Value::Nil => 0,
        Value::String(string) => match string.parse() {
            Ok(integer) => integer,
            Err(_) => return Err(RuntimeError {kind: RuntimeErrorType::ParseError(string, "Integer".to_string())}.into())
        },
        _ => todo!(),
    };
    Ok(Value::Integer(result))
}

pub fn to_float(mut arguments: Vec<Value>) -> Result<Value, Error> {
    if arguments.is_empty() {
        return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("Float".to_string())}.into())
    } else if arguments.len() > 1 {
        todo!()
    }
    let value = arguments.pop().unwrap();
    let result = match value {
        Value::Float(float) => float,
        Value::Integer(integer) => integer as f64,
        Value::Boolean(_) => return Err(RuntimeError {kind: RuntimeErrorType::ParseError("Boolean".to_string(), "Float".to_string())}.into()),
        Value::Nil => return Err(RuntimeError {kind: RuntimeErrorType::ParseError("Nil".to_string(), "Float".to_string())}.into()),
        Value::String(string) => match string.parse() {
            Ok(float) => float,
            Err(_) => return Err(RuntimeError {kind: RuntimeErrorType::ParseError(string, "Float".to_string())}.into())
        },
        _ => todo!(),
    };
    Ok(Value::Float(result))
}

pub fn to_boolean(arguments: Vec<Value>) -> Result<Value, Error> {
    if arguments.is_empty() {
        return Err(RuntimeError {kind: RuntimeErrorType::InsufficientOperands("Boolean".to_string())}.into())
    } else if arguments.len() > 1 {
        todo!()
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

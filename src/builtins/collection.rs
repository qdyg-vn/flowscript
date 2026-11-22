use crate::error_handler::{Error, RuntimeError, RuntimeErrorType, TypeError, TypeErrorType};
use crate::value::Value;

pub fn length(arguments: Vec<Value>) -> Result<Value, Error> {
    if arguments.len() != 1 {
        return Err(TypeError{ kind: TypeErrorType::ArityMismatch { function: "length".to_string(), expected_arity: 1, found_arity: arguments.len() } }.into())
    }
    match &arguments[0] {
        Value::Array(array) => Ok(Value::Integer(array.len() as i64)),
        Value::String(string) => Ok(Value::Integer(string.len() as i64)),
        something => Err(TypeError { kind: TypeErrorType::NotSequence(something.clone())}.into())
    }
}

pub fn push(mut arguments: Vec<Value>) -> Result<Value, Error> {
    if arguments.len() < 2 { return Err(RuntimeError { kind: RuntimeErrorType::InsufficientOperands { function: "push".to_string(), min_arity: 2 } }.into()) }
    let value = arguments.remove(0);
    let mut array = match value {
        Value::Array(array) => array,
        _ => todo!()
    };
    arguments.into_iter().for_each(|element| array.push(element));
    Ok(Value::Array(array))
}

pub fn pop(mut arguments: Vec<Value>) -> Result<Value, Error> {
    if arguments.len() != 1 { return Err(TypeError{ kind: TypeErrorType::ArityMismatch { function: "pop".to_string(), expected_arity: 1, found_arity: arguments.len() } }.into()) }
    let value = arguments.pop().unwrap();
    let mut array = match value {
        Value::Array(array) => array,
        _ => todo!()
    };
    array.pop();
    Ok(Value::Array(array))
}

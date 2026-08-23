use crate::error_handler::{Error, TypeError, TypeErrorType};
use crate::value::Value;

pub fn length(arguments: Vec<Value>) -> Result<Value, Error> {
    if arguments.is_empty() {
        todo!()
    } else if arguments.len() > 1 {
        todo!()
    }
    match &arguments[0] {
        Value::Array(array) => Ok(Value::Integer(array.len() as i64)),
        Value::String(string) => Ok(Value::Integer(string.len() as i64)),
        something => Err(TypeError {kind: TypeErrorType::NotSequence(something.clone())}.into())
    }
}

pub fn push(mut arguments: Vec<Value>) -> Result<Value, Error> {
    if arguments.len() < 2 { todo!() }
    let value = arguments.pop().unwrap();
    let mut array = match value {
        Value::Array(array) => array,
        _ => todo!()
    };
    arguments.into_iter().for_each(|element| array.push(element));
    Ok(Value::Array(array))
}

pub fn pop(mut arguments: Vec<Value>) -> Result<Value, Error> {
    if arguments.len() != 1 { todo!() }
    let value = arguments.pop().unwrap();
    let mut array = match value {
        Value::Array(array) => array,
        _ => todo!()
    };
    array.pop();
    Ok(Value::Array(array))
}

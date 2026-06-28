use crate::error_handler::{Error, TypeError, TypeErrorType};
use crate::value::Value;

pub fn length(arguments: &[Value]) -> Result<Value, Error> {
    match &arguments[0] {
        Value::Array(array) => Ok(Value::Integer(array.len() as i64)),
        Value::String(string) => Ok(Value::Integer(string.len() as i64)),
        something => Err(TypeError {kind: TypeErrorType::NotSequence(something.clone())}.into())
    }
}
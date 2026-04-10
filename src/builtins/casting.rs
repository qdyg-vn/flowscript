use crate::value::Value;
use std::rc::Rc;

pub fn to_string(arguments: &[Value]) -> Value {
    if arguments.len() == 0 {
        return Value::Nil
    }
    let result = match &arguments[0] {
        Value::Float(float) => float.to_string(),
        Value::Integer(integer) => integer.to_string(),
        Value::Boolean(boolean) => boolean.to_string(),
        Value::Nil => "Nil".to_string(),
        Value::String(_) => return arguments[0].clone(),
        _ => todo!(),
    };
    Value::String(Rc::from(result))
}

use crate::value::Value;

pub fn to_string(arguments: &[Value]) -> Value {
    if arguments.len() == 0 {
        return Value::Nil
    }
    let result = match &arguments[0] {
        Value::Boolean(boolean) => boolean.to_string(),
        Value::Nil => "Nil".to_string(),
        Value::Float(float) => float.to_string(),
        Value::String(string) => string.clone(),
        Value::Integer(integer) => integer.to_string(),
    };
    Value::String(result)
}

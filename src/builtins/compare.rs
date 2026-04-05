use crate::value::Value;

pub fn equal(arguments: &[Value]) -> Value {
    let result = match (arguments[0].clone(), arguments[1].clone()) {
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Integer(a), Value::Integer(b)) => a == b,
        _ => false
    };
    Value::Boolean(result)
}
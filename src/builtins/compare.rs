use crate::value::Value;

pub fn equal(arguments: &[Value]) -> Value {
    let result = match (arguments[0].clone(), arguments[1].clone()) {
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        _ => false
    };
    Value::Boolean(result)
}

pub fn less(arguments: &[Value]) -> Value {
    let result = match (arguments[0].clone(), arguments[1].clone()) {
        (Value::Float(a), Value::Float(b)) => a < b,
        (Value::Integer(a), Value::Integer(b)) => a < b,
        (Value::Boolean(a), Value::Boolean(b)) => a < b,
        (Value::String(a), Value::String(b)) => a < b,
        _ => false
    };
    Value::Boolean(result)
}

pub fn greater(arguments: &[Value]) -> Value {
    let result = match (arguments[0].clone(), arguments[1].clone()) {
        (Value::Float(a), Value::Float(b)) => a > b,
        (Value::Integer(a), Value::Integer(b)) => a > b,
        (Value::Boolean(a), Value::Boolean(b)) => a > b,
        (Value::String(a), Value::String(b)) => a > b,
        _ => false
    };
    Value::Boolean(result)
}

pub fn not_equal(arguments: &[Value]) -> Value {
    let result = match (arguments[0].clone(), arguments[1].clone()) {
        (Value::Float(a), Value::Float(b)) => a != b,
        (Value::Integer(a), Value::Integer(b)) => a != b,
        (Value::Boolean(a), Value::Boolean(b)) => a != b,
        (Value::String(a), Value::String(b)) => a != b,
        _ => false
    };
    Value::Boolean(result)
}

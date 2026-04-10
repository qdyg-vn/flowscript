use crate::value::Value;
use std::rc::Rc;

pub fn add(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() == 0 { return Err("Addition requires at least one operand".to_string()) }
    let (first, rest) = arguments.split_first().unwrap();
    if rest.is_empty() { return Ok(first.to_owned()) };
    rest.iter().try_fold(first.clone(), |accumulator, x| {
        match (&accumulator, x) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(Rc::from(a.to_string() + b))),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            _ => Err(format!("Cannot add {:?} and {:?}", accumulator, x))
        }
    })
}

pub fn minus(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() == 0 { return Err("Minus requires at least one operand".into()) }
    let (first, rest) = arguments.split_first().unwrap();
    if rest.is_empty() {
        return match first {
            Value::Integer(a) => Ok(Value::Integer(-a)),
            Value::Float(a) => Ok(Value::Float(-a)),
            _ => Err("Unary minus only supports numeric types".into())
        }
    };
    rest.iter().try_fold(first.clone(), |accumulator, x| {
        match (&accumulator, x) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a.replacen(&**b, "", 1).into())),
            _ => Err(format!("Cannot subtract {:?} and {:?}", accumulator, x))
        }
    })
}

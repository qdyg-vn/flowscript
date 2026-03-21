use crate::value::Value;

pub fn add(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() == 0 { return Err("Addition requires at least one operand".to_string()) }
    else if arguments.len() == 1 { return Ok(arguments.get(0).unwrap().to_owned()) };
    let (first, rest) = arguments.split_first().ok_or("Failed to get first argument".to_string())?;
    rest.iter().try_fold(first.clone(), |accumulator, x| {
        match (accumulator, x) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            _ => Err("Mismatched types in addition function".into())
        }
    })
}
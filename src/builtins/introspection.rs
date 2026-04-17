use crate::value::Value;

pub fn len(arguments: &[Value]) -> Value {
    match arguments {
        [Value::Array(elements)] => { Value::Integer((*elements).len() as i64) },
        _ => todo!()
    }
}
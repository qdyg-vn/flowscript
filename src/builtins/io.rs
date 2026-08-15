use crate::value::Value;

pub fn print(item: &[Value]) {
    if item.is_empty() {
        println!()
    } else {
        println!("{}", item[0])
    }
}
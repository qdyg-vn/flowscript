use crate::value::Value;

pub fn print(item: Vec<Value>) {
    if item.is_empty() {
        println!()
    } else {
        println!("{}", item[0])
    }
}
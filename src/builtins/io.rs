use crate::value::Value;

pub fn print(item: &[Value]) {
    if item.len() == 0 {
        println!()
    } else {
        println!("{}", item[0])
    }
}
use crate::value::Value;
use std::collections::{HashMap};

#[derive(Default, Debug)]
pub struct ConstantsPool {
    pub constants: Vec<Value>,
    pub lookup: HashMap<Value, usize>,
}

impl ConstantsPool {
    pub fn add_constant(&mut self, constant: Value) -> usize {
        if matches!(constant, Value::Function(_) | Value::Closure(_)) {
            let index = self.constants.len();
            self.constants.push(constant);
            return index
        }
        if let Some(&index) = self.lookup.get(&constant) {
            return index
        }
        let index = self.constants.len();
        self.constants.push(constant.clone());
        self.lookup.insert(constant, index);
        index
    }
}

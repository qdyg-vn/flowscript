use crate::value::{HeavyValue, Value};
use std::collections::{HashMap};

#[derive(Default, Debug)]
pub struct ConstantsPool {
    pub constants: Vec<Value>,
    pub lookup: HashMap<Value, usize>,
    pub heavy_constants: Vec<HeavyValue>,
    pub heavy_lookup: HashMap<HeavyValue, usize>,
}

impl ConstantsPool {
    pub fn add_constant(&mut self, constant: Value) -> usize {
        if let Some(&index) = self.lookup.get(&constant) {
            return index
        }
        let index = self.constants.len();
        self.constants.push(constant.clone());
        self.lookup.insert(constant, index);
        index
    }

    pub fn add_heavy_constant(&mut self, constant: HeavyValue) -> usize {
        if matches!(constant, HeavyValue::Function(_) | HeavyValue::Closure(_)) {
            let heavy_index = self.heavy_constants.len();
            self.heavy_constants.push(constant);
            return heavy_index
        };
        let heavy_index = match self.heavy_lookup.get(&constant) {
            Some(&heavy_index) => heavy_index,
            None => {
                let heavy_index = self.heavy_constants.len();
                self.heavy_constants.push(constant.clone());
                self.heavy_lookup.insert(constant.clone(), heavy_index);
                heavy_index
            }
        };
        let index = self.add_constant(match constant {
            HeavyValue::String(_) => Value::StringPointer(heavy_index as u32),
            HeavyValue::Array(_) => Value::ArrayPointer(heavy_index as u32),
            _ => unreachable!()
        });
        index
    }
}

use crate::value::{HeavyValue, LightValue};
use std::collections::{HashMap};
use crate::instructions::Chunk;

#[derive(Default, Debug)]
pub struct ConstantsPool {
    pub constants: Vec<LightValue>,
    pub lookup: HashMap<LightValue, usize>,
    pub heavy_constants: Vec<HeavyValue>,
    pub heavy_lookup: HashMap<HeavyValue, usize>,
    pub functions: Vec<Vec<Chunk>>,
}

impl ConstantsPool {
    pub fn new(define_function_count: usize) -> Self {
        Self { functions: vec![vec![]; define_function_count], ..Self::default() }
    }

    pub fn add_constant(&mut self, constant: LightValue) -> usize {
        if let Some(&index) = self.lookup.get(&constant) {
            return index
        }
        let index = self.constants.len();
        self.constants.push(constant);
        self.lookup.insert(constant, index);
        index
    }

    pub fn write_function_body(&mut self, index: usize, body: Vec<Chunk>) {
        self.functions[index] = body
    }

    pub fn add_heavy_constant(&mut self, constant: &HeavyValue) -> usize {
        let heavy_index = match self.heavy_lookup.get(constant) {
            Some(&heavy_index) => heavy_index,
            None => {
                let heavy_index = self.heavy_constants.len();
                self.heavy_constants.push(constant.clone());
                self.heavy_lookup.insert(constant.clone(), heavy_index);
                heavy_index
            }
        };
        self.add_constant(LightValue::StringPointer(heavy_index as u32))
    }
}

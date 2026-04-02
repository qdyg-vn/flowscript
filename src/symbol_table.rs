use crate::builtins::BUILTIN_TABLE;
use crate::error_handler::Error;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum SymbolType {
    Scope(u16, u16),
    Builtin(u16)
}

pub struct SymbolTable {
    pub builtins: HashMap<String, u16>,
    pub scopes: Vec<HashMap<String, SymbolType>>
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut builtins = HashMap::new();
        for (index, &function) in BUILTIN_TABLE.iter().enumerate() {
            builtins.insert(function.name.to_string(), index as u16);
        }
        Self {
            builtins,
            scopes: Vec::new()
        }
    }

    pub fn resolve(&self, name: &str) -> Result<SymbolType, Error> {
        if let Some(&index) = self.builtins.get(name) {
            return Ok(SymbolType::Builtin(index))
        }
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Ok(symbol.clone())
            }
        }
        todo!("Error at here! Because we can't find it")
    }
}
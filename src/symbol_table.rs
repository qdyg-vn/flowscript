use crate::builtins::BUILTIN_TABLE;
use crate::error_handler::Error;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum SymbolType {
    Global(u16),
    Local(u16, u16),
    Builtin(u16)
}

pub struct Symbol {
    pub name: u16,
    pub symbol_type: SymbolType
}

pub struct SymbolTable {
    pub builtins: HashMap<String, u16>,
    pub scopes: Vec<HashMap<String, Symbol>>
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
                return Ok(symbol.symbol_type.clone())
            }
        }
        todo!("Error at here! Because we can't find it")
    }
}
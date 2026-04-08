use crate::builtins::BUILTIN_TABLE;
use crate::error_handler::Error;
use std::collections::{HashMap, hash_map::Entry};

#[derive(Clone, Copy, Debug)]
pub enum SymbolType {
    Scope(u16, u16),
    Builtin(u16),
}

#[derive(Debug)]
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

    pub fn new_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn add_variable(&mut self, variable: String) -> Result<SymbolType, SymbolType> {
        let scope = self.scopes.len() as u16 - 1;
        let last_scope = self.scopes.last_mut().unwrap();
        let index = last_scope.len() as u16;
        match last_scope.entry(variable) {
            Entry::Vacant(entry) => {
                let new_symbol = SymbolType::Scope(scope, index);
                entry.insert(new_symbol);
                Ok(new_symbol)
            }
            Entry::Occupied(entry) => Err(*entry.get())
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
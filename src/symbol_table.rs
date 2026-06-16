use crate::builtins::BUILTIN_TABLE;
use crate::error_handler::{SemanticError, SemanticErrorType};
use std::collections::{HashMap, hash_map::Entry};
use crate::value::VariableType;

#[derive(Debug, Clone, Copy)]
pub struct FunctionSignature {
    pub start: u32,
    pub length: u8,
}

#[derive(Copy, Clone, Debug)]
pub enum SymbolType {
    Scope(u16, u16, VariableType),
    Builtin(u16),
}

#[derive(Debug)]
pub struct SymbolTable {
    pub builtins: HashMap<String, u16>,
    pub scopes: Vec<HashMap<String, SymbolType>>,
    pub functions: Vec<FunctionSignature>,
    pub all_arguments: Vec<VariableType>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut builtins = HashMap::new();
        for (index, &function) in BUILTIN_TABLE.iter().enumerate() {
            builtins.insert(function.name.to_string(), index as u16);
        }
        Self {
            builtins,
            scopes: vec![HashMap::new()],
            functions: Vec::new(),
            all_arguments: Vec::new(),
        }
    }

    pub fn new_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn add_variable(&mut self, variable: String, mut variable_type: VariableType) -> Result<SymbolType, SymbolType> {
        if let VariableType::Function(arguments_length) = variable_type {
            variable_type = VariableType::Function(self.functions.len() as u32);
            self.functions.push(FunctionSignature {start: self.all_arguments.len() as u32, length: arguments_length as u8})
        }
        let scope = self.scopes.len() as u16 - 1;
        let last_scope = self.scopes.last_mut().unwrap();
        let index = last_scope.len() as u16;
        match last_scope.entry(variable) {
            Entry::Vacant(entry) => {
                let new_symbol = SymbolType::Scope(scope, index, variable_type);
                entry.insert(new_symbol);
                Ok(new_symbol)
            }
            Entry::Occupied(entry) => Err(*entry.get())
        }
    }

    pub fn resolve(&self, name: &str) -> Result<SymbolType, SemanticError> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Ok(*symbol)
            }
        }
        if let Some(&index) = self.builtins.get(name) {
            return Ok(SymbolType::Builtin(index))
        }
        Err(SemanticError {kind: SemanticErrorType::UndefinedIdentifier(name.into())})
    }

    pub fn get_arguments(&self, function_index: u32) -> Vec<VariableType> {
        let function = &self.functions[function_index as usize];
        self.all_arguments[function.start as usize..function.start as usize + function.length as usize].to_vec()
    }
}
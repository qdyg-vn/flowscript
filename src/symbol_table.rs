use crate::builtins::BUILTIN_TABLE;
use crate::error_handler::{SemanticError, SemanticErrorType};
use std::collections::{HashMap, hash_map::Entry};
use crate::value::Kind;

#[derive(Debug, Clone, Copy)]
pub struct FunctionSignature {
    pub start: u32,
    pub length: u8,
    pub result: Kind,
}

#[derive(Debug, Clone, Copy)]
pub struct RelativeReference {
    pub index_in_stations: u16,
    pub variable_index: u32,
}

#[derive(Copy, Clone, Debug)]
pub enum SymbolType {
    VariableScope(u16, u32),
    FunctionScope(u32),
    Builtin(u16),
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    builtins: HashMap<String, u16>,
    scopes: Vec<HashMap<String, SymbolType>>,
    pub all_variable: Vec<Kind>,
    functions: Vec<FunctionSignature>,
    pub all_parameters: Vec<Kind>,
    pub pipeline: Vec<RelativeReference>,
}

impl SymbolTable {
    pub fn with_builtins() -> Self {
        let mut table = Self {
            scopes: vec![HashMap::new()],
            ..Self::default()
        };
        for (index, &function) in BUILTIN_TABLE.iter().enumerate() {
            table.builtins.insert(function.name.to_string(), index as u16);
        }
        table
    }

    pub fn new_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn add_variable(&mut self, variable: String, kind: Kind) -> Result<SymbolType, SymbolType> {
        let scope = self.scopes.len() as u16 - 1;
        let last_scope = self.scopes.last_mut().unwrap();
        let index = last_scope.len() as u16;
        match last_scope.entry(variable) {
            Entry::Vacant(entry) => {
                let variable_index = self.all_variable.len() as u32;
                self.all_variable.push(kind);
                let new_symbol = SymbolType::VariableScope(index, variable_index);
                entry.insert(new_symbol);
                Ok(new_symbol)
            }
            Entry::Occupied(entry) => Err(*entry.get())
        }
    }

    pub fn add_relative_reference(&mut self, index_in_stations: u16, kind: Kind) -> SymbolType {
        for (index, relative_reference) in self.pipeline.iter().enumerate() {
            if relative_reference.index_in_stations == index_in_stations {
                return SymbolType::VariableScope(index as u16, relative_reference.variable_index)
            }
        };
        let index = self.pipeline.len() as u16;
        let variable_index = self.all_variable.len() as u32;
        self.pipeline.push(RelativeReference { index_in_stations, variable_index });
        self.all_variable.push(kind);
        SymbolType::VariableScope(index, variable_index)
    }

    pub fn add_function(&mut self, variable: String, signature_length: u32, result: Kind) -> Result<SymbolType, SymbolType> {
        let signature_index = self.functions.len() as u32;
        let last_scope = self.scopes.last_mut().unwrap();
        match last_scope.entry(variable) {
            Entry::Vacant(entry) => {
                self.functions.push(FunctionSignature { start: self.all_parameters.len() as u32, length: signature_length as u8, result });
                let new_symbol = SymbolType::FunctionScope(signature_index);
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

    pub fn get_parameters(&self, function_index: u32) -> Vec<Kind> {
        let function = self.functions[function_index as usize];
        self.all_parameters[function.start as usize..function.start as usize + function.length as usize].to_vec()
    }

    pub fn get_result(&self, function_index: u32) -> Kind {
        let function = self.functions[function_index as usize];
        function.result
    }
}
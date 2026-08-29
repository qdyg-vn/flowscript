use crate::builtins::BUILTIN_TABLE;
use crate::error_handler::{SemanticError, SemanticErrorType, TypeError, TypeErrorType};
use std::collections::{HashMap, hash_map::Entry};
use crate::value::Kind;

#[derive(Debug, Hash, Eq, PartialEq)]
struct FunctionSignature {
    id: u32,
    parameters_kind: Vec<Kind>,
}

#[derive(Debug, Copy, Clone)]
pub struct Function {
    pub index: u32,
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
    functions: HashMap<FunctionSignature, Function>,
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

    pub fn add_function(&mut self, function_name: String, parameters_kind: Vec<Kind>, result: Kind) {
        let function_index = self.functions.len() as u32;
        let last_scope = self.scopes.last_mut().unwrap();
        match last_scope.entry(function_name) {
            Entry::Vacant(entry) => {
                let function_signature = FunctionSignature { id: function_index, parameters_kind };
                let function = Function { index: function_index, result };
                self.functions.insert(function_signature, function);
                let new_symbol = SymbolType::FunctionScope(function_index);
                entry.insert(new_symbol);
            }
            Entry::Occupied(entry) => {
                let SymbolType::FunctionScope(id) = entry.get() else { todo!() };
                let function_signature = FunctionSignature { id: *id, parameters_kind };
                match self.functions.entry(function_signature) {
                    Entry::Vacant(entry) => { entry.insert(Function { index: function_index, result }); },
                    Entry::Occupied(_) => todo!(),
                }
            }
        }
    }

    pub fn find_function(&self, id: u32, parameters_kind: Vec<Kind>) -> Result<Function, TypeError> {
        let function = FunctionSignature { id, parameters_kind: parameters_kind.clone() };
        match self.functions.get(&function) {
            Some(function_index) => Ok(*function_index),
            None => Err(TypeError { kind: TypeErrorType::NoFunctionFound(parameters_kind) }),
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
}
use crate::builtins::BUILTIN_TABLE;
use crate::error_handler::{Error, SemanticError, SemanticErrorType, TypeError, TypeErrorType};
use std::collections::{hash_map::Entry, HashMap};
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

#[derive(Debug)]
struct Structure {
    field_kind_indices: HashMap<String, u8>,
    field_kinds: Vec<Kind>,
    fields_offset: Vec<u16>,
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
    StructScope(u16),
    Builtin(u16),
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    builtins: HashMap<String, u16>,
    scopes: Vec<HashMap<String, SymbolType>>,
    pub all_variable: Vec<Kind>,
    functions: HashMap<FunctionSignature, Function>,
    structures: Vec<Structure>,
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

    pub fn add_struct(&mut self, name: String, field_kind_indices: HashMap<String, u8>, field_kinds: Vec<Kind>) -> Option<SemanticError> {
        let fields_offset = self.compute_field_offsets(&field_kinds);
        let last_scope = self.scopes.last_mut().unwrap();
        match last_scope.entry(name) {
            Entry::Vacant(entry) => {
                let index = self.structures.len() as u16;
                self.structures.push(Structure { field_kind_indices, field_kinds, fields_offset });
                let new_symbol = SymbolType::StructScope(index);
                entry.insert(new_symbol);
                None
            }
            Entry::Occupied(_) => Some(SemanticError { kind: SemanticErrorType::DuplicateStructDefinition })
        }
    }

    fn compute_field_offsets(&self, field_kinds: &[Kind]) -> Vec<u16> {
        let mut current_offset = 0;
        field_kinds.iter().map(|&kind| {
            let offset = current_offset;
            current_offset += kind.size();
            offset
        }).collect()
    }

    pub fn resolve_struct(&self, struct_name: String, struct_index: u16, field_names: &Vec<String>) -> Result<Vec<u8>, Vec<Error>> {
        let structure = &self.structures[struct_index as usize];
        let mut field_kind_indices = Vec::new();
        let mut errors = Vec::new();
        for field_name in field_names {
            if let Some(index) = structure.field_kind_indices.get(field_name) {
                field_kind_indices.push(*index);
            } else {
                errors.push(SemanticError { kind: SemanticErrorType::NoFieldFound(struct_name.clone(), field_name.clone()) }.into());
            }
        }
        for field_name in structure.field_kind_indices.keys() {
            if !field_names.contains(field_name) {
                errors.push(SemanticError { kind: SemanticErrorType::MissingField(struct_name.clone(), field_name.clone()) }.into());
            }
        }
        if errors.is_empty() { Ok(field_kind_indices) } else { Err(errors) }
    }

    pub fn find_struct_index(&self, variable_name: String, variable_index: u32) -> Result<u16, Error> {
        let Kind::Struct(struct_index) = self.all_variable[variable_index as usize] else { return Err(TypeError { kind: TypeErrorType::NotAStruct(variable_name) }.into()) };
        Ok(struct_index)
    }

    pub fn find_field_index(&self, struct_name: String, struct_index: u16, field_name: &str) -> Result<u8, Error> {
        let structure = &self.structures[struct_index as usize];
        match structure.field_kind_indices.get(field_name) {
            Some(index) => Ok(*index),
            None => Err(SemanticError { kind: SemanticErrorType::NoFieldFound(struct_name, field_name.to_string()) }.into()),
        }
    }
    
    pub fn get_field_offset(&self, struct_index: u16, field_index: u8) -> u16 {
        let structure = &self.structures[struct_index as usize];
        structure.fields_offset[field_index as usize]
    }
    
    pub fn get_field_kind(&self, struct_index: u16, field_index: u8) -> Kind {
        let structure = &self.structures[struct_index as usize];
        structure.field_kinds[field_index as usize]
    }

    pub fn check_struct_fields(&self, struct_index: u16, field_kind_indices: &[u8], field_kinds: &[Kind]) -> Vec<Error> {
        let mut errors = Vec::with_capacity(field_kind_indices.len());
        let structure = &self.structures[struct_index as usize];
        for (index, field_kind) in field_kind_indices.iter().zip(field_kinds) {
            let expected_kind = structure.field_kinds[*index as usize];
            if expected_kind != *field_kind {
                errors.push(TypeError { kind: TypeErrorType::TypeMismatch(expected_kind, *field_kind) }.into())
            }
        }
        errors
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

    pub fn add_function(&mut self, function_name: String, parameters_kind: Vec<Kind>, result: Kind) -> Option<SemanticError> {
        let function_index = self.functions.len() as u32;
        let last_scope = self.scopes.last_mut().unwrap();
        match last_scope.entry(function_name) {
            Entry::Vacant(entry) => {
                let function_signature = FunctionSignature { id: function_index, parameters_kind };
                let function = Function { index: function_index, result };
                self.functions.insert(function_signature, function);
                let new_symbol = SymbolType::FunctionScope(function_index);
                entry.insert(new_symbol);
                None
            }
            Entry::Occupied(entry) => {
                let SymbolType::FunctionScope(id) = entry.get() else { todo!() };
                let function_signature = FunctionSignature { id: *id, parameters_kind };
                match self.functions.entry(function_signature) {
                    Entry::Vacant(entry) => { entry.insert(Function { index: function_index, result }); None },
                    Entry::Occupied(_) => Some(SemanticError { kind: SemanticErrorType::DuplicateFunctionDefinition }),
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
        Err(SemanticError { kind: SemanticErrorType::UndefinedIdentifier(name.into())})
    }
}
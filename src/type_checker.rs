use crate::builtins::{get_builtin, Signature};
use crate::error_handler::{Error, ErrorHandler, TypeError, TypeErrorType};
use crate::node::{ResolvedNode, TypedAST, TypedNode, AST, ConditionBranch};
use crate::value::Kind;
use crate::builtins::get_types;
use crate::symbol_table::SymbolTable;

pub struct TypeChecker {
    error_handler: ErrorHandler,
    symbol_table: SymbolTable,
}

impl TypeChecker {
    pub fn new(error_handler: ErrorHandler, symbol_table: SymbolTable) -> Self {
        Self { error_handler, symbol_table }
    }

    pub fn checker(mut self, ast: AST) -> (ErrorHandler, TypedAST) {
        let mut typed_nodes = Vec::with_capacity(ast.nodes.len());
        for station in ast.nodes {
            match self.check(&station, &typed_nodes) {
                Ok(station) => typed_nodes.push(station),
                Err(errors) => self.error_handler.errors.extend(errors),
            }
        }
        if !self.error_handler.errors.is_empty() { self.error_handler.report_exit() }
        (self.error_handler, TypedAST { nodes: typed_nodes, arity: ast.arity, variables_count: ast.variables_count, max_relative_reference: ast.max_relative_reference })
    }

    pub fn check(&mut self, station: &ResolvedNode, typed_stations: &[TypedNode]) -> Result<TypedNode, Vec<Error>> {
        match station {
            ResolvedNode::Literal(value) => Ok(TypedNode::Literal(*value)),
            ResolvedNode::HeavyLiteral(value) => Ok(TypedNode::HeavyLiteral(value.clone())),
            ResolvedNode::BuiltinCall { index, arguments } => {
                let types = get_types(*index);
                let typed_arguments: Vec<TypedNode> = arguments.iter().map(|argument| self.check(argument, typed_stations)).collect::<Result<_, _>>()?;
                let arguments_kind: Vec<Kind> = typed_arguments.iter().map(|argument| self.find_node_kind(argument)).collect();
                match self.find_result_kind(&arguments_kind, types) {
                    Ok(result_kind) => {
                        let mut node = TypedNode::BuiltinCall { index: *index, arguments: typed_arguments, result: result_kind };
                        self.rewrite_builtin_function(&mut node);
                        Ok(node)
                    },
                    Err(errors) => Err(errors),
                }
            },
            ResolvedNode::Call { arguments, function_index } => {
                let typed_arguments: Vec<TypedNode> = arguments.iter().map(|argument| self.check(argument, typed_stations)).collect::<Result<_, _>>()?;
                let arguments_kind = typed_arguments.iter().map(|argument| self.find_node_kind(argument)).collect();
                match self.symbol_table.find_function(*function_index, arguments_kind) {
                    Ok(function) => Ok(TypedNode::Call { function_index: function.index, arguments: typed_arguments, result: function.result }),
                    Err(errors) => Err(vec![errors.into()]),
                }
            },
            ResolvedNode::Pipeline(stations) => {
                let mut typed_stations = Vec::with_capacity(stations.len());
                self.symbol_table.pipeline.clear();
                for station in stations {
                    let station = self.check(station, &typed_stations)?;
                    typed_stations.push(station)
                }
                Ok(TypedNode::Pipeline(typed_stations))
            },
            ResolvedNode::RelativeReference(x, y, variable_index) => {
                let kind = self.symbol_table.all_variable[*variable_index as usize];
                Ok(TypedNode::RelativeReference(*x, *y, kind))
            },
            ResolvedNode::StationCapture(index, variable_index) => {
                let received_kind = self.find_node_kind(typed_stations.last().unwrap());
                self.symbol_table.all_variable[*variable_index as usize] = received_kind;
                Ok(TypedNode::StationCapture(*index, received_kind))
            },
            ResolvedNode::Variable(index, variable_index) => {
                let kind = self.symbol_table.all_variable[*variable_index as usize];
                Ok(TypedNode::Variable(*index, kind))
            },
            ResolvedNode::SoftAssignment(index, variable_index) => {
                let received_kind = self.find_node_kind(typed_stations.last().unwrap());
                self.symbol_table.all_variable[*variable_index as usize] = received_kind;
                Ok(TypedNode::Assignment(*index, received_kind))
            },
            ResolvedNode::Assignment(index, variable_index, kind) => {
                let received_kind = self.find_node_kind(typed_stations.last().unwrap());
                if received_kind != *kind { self.error_handler.push_error(TypeError { kind: TypeErrorType::AssignTypeMismatch(received_kind, *kind) }) }
                self.symbol_table.all_variable[*variable_index as usize] = *kind;
                Ok(TypedNode::Assignment(*index, *kind))
            }
            ResolvedNode::DefineFunction { function_index, body } => {
                let child_typed_stations = Vec::with_capacity(body.nodes.len());
                Ok(TypedNode::DefineFunction {
                    function_index: *function_index,
                    body: TypedAST {
                        nodes: body.nodes.iter().map(|node| self.check(node, &child_typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?,
                        arity: body.arity,
                        variables_count: body.variables_count,
                        max_relative_reference: body.max_relative_reference
                    }
                })
            },
            ResolvedNode::Condition { branches, final_branch } => {
                let mut typed_branches = Vec::with_capacity(branches.len());
                for branch in branches {
                    typed_branches.push(ConditionBranch { condition: branch.condition.iter().map(|condition| self.check(condition, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?, body: branch.body.iter().map(|body| self.check(body, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()? })
                };
                Ok(TypedNode::Condition { branches: typed_branches, final_branch: final_branch.iter().map(|body| self.check(body, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()? } )
            },
            ResolvedNode::Return(value) => Ok(TypedNode::Return(Box::new(self.check(value, typed_stations)?))),
            ResolvedNode::Array(elements) => Ok(TypedNode::Array(elements.iter().map(|element| self.check(element, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?)),
            ResolvedNode::StructInstantiation { index, field_kind_indices, field_values } => {
                let typed_field_values: Vec<TypedNode> = field_values.iter().map(|field_value| self.check(field_value, typed_stations)).collect::<Result<_, _>>()?;
                let field_kinds: Vec<Kind> = typed_field_values.iter().map(|argument| self.find_node_kind(argument)).collect();
                self.error_handler.errors.extend(self.symbol_table.check_struct_fields(*index, field_kind_indices, &field_kinds));
                Ok(TypedNode::StructInstantiation { field_kind_indices: field_kind_indices.clone(), field_values: typed_field_values, structure: Kind::Struct(*index) })
            }
            ResolvedNode::FieldAccess { struct_name, index, variable_index, fields_name } => {
                let mut struct_index = match self.symbol_table.find_struct_index(struct_name, *variable_index) {
                    Ok(index) => index,
                    Err(error) => { 
                        self.error_handler.push_error(error);
                        return Ok(TypedNode::FieldAccess { index: 0, fields_offset: vec![], field_kind: Kind::Undefined })
                    },
                };
                let mut fields_offset = Vec::new();
                let mut field_kind = Kind::Undefined;
                for field_name in fields_name {
                    let field_index = match self.symbol_table.find_field_index(struct_name, struct_index, field_name) {
                        Ok(index) => index,
                        Err(error) => {
                            self.error_handler.push_error(error);
                            return Ok(TypedNode::FieldAccess { index: struct_index, fields_offset: vec![], field_kind: Kind::Undefined })
                        },
                    };
                    let field_offset = self.symbol_table.get_field_offset(struct_index, field_index);
                    fields_offset.push(field_offset);
                    field_kind = self.symbol_table.get_field_kind(struct_index, field_index);
                    if let Kind::Struct(index) = field_kind {
                        struct_index = index;
                    }
                }
                Ok(TypedNode::FieldAccess { index: *index, fields_offset, field_kind })
            },
        }
    }

    fn find_result_kind(&mut self, arguments_kind: &[Kind], kinds: &[Signature]) -> Result<Kind, Vec<Error>> {
        let mut errors = Vec::with_capacity(arguments_kind.len());
        let mut mismatch_count = i32::MAX;
        for signature in kinds {
            let mut signature_mismatch_count = 0;
            let mut signature_errors = Vec::with_capacity(arguments_kind.len());
            if signature.infinite_arity {
                if arguments_kind.len() < signature.min_arity as usize {
                    signature_mismatch_count += 1;
                    signature_errors.push(TypeError{ kind: TypeErrorType::ArityMismatch(arguments_kind.len(), signature.min_arity as usize) }.into());
                } else {
                    for argument_kind in arguments_kind {
                        if *argument_kind != Kind::Undefined && *argument_kind != signature.arguments[0] {
                            signature_mismatch_count += 1;
                            signature_errors.push(TypeError { kind: TypeErrorType::TypeMismatch(signature.arguments[0], *argument_kind) }.into());
                        }
                    }
                }
            } else {
                if arguments_kind.len() != signature.min_arity as usize {
                    signature_mismatch_count += 1;
                    signature_errors.push(TypeError{ kind: TypeErrorType::ArityMismatch(arguments_kind.len(), signature.min_arity as usize) }.into());
                } else {
                    for (&argument_kind, &required_kind) in arguments_kind.iter().zip(signature.arguments) {
                        if argument_kind != Kind::Undefined && argument_kind != required_kind {
                            signature_mismatch_count += 1;
                            signature_errors.push(TypeError { kind: TypeErrorType::TypeMismatch(required_kind, argument_kind) }.into());
                        }
                    }
                }
            }
            if signature_mismatch_count == 0 {
                return Ok(signature.result)
            }
            if signature_mismatch_count <= mismatch_count {
                mismatch_count = signature_mismatch_count;
                errors = signature_errors;
            }
        }
        Err(errors)
    }

    fn find_node_kind(&self, node: &TypedNode) -> Kind {
        match node {
            TypedNode::Literal(value) => value.get_kind(),
            TypedNode::HeavyLiteral(value) => value.get_kind(),
            TypedNode::Variable(_, kind) | TypedNode::Assignment(_, kind) => *kind,
            TypedNode::BuiltinCall { result, ..} | TypedNode::Call { result, .. } => *result,
            TypedNode::RelativeReference(_, _, parent_kind) => *parent_kind,
            TypedNode::Add(_, _, result) | TypedNode::Minus(_, _, result)
            | TypedNode::Multiply(_, _, result) | TypedNode::Equal(_, _, result)
            | TypedNode::LessThan(_, _, result) | TypedNode::GreaterThan(_, _, result)
            | TypedNode::LessThanOrEqual(_, _, result) | TypedNode::GreaterThanOrEqual(_, _, result)
            | TypedNode::NotEqual(_, _, result) => *result,
            TypedNode::Array(_) => Kind::Array,
            TypedNode::StructInstantiation { structure, .. } => *structure,
            TypedNode::FieldAccess { field_kind, .. } => *field_kind,
            _ => {todo!("Currently under development")}
        }
    }

    fn rewrite_builtin_function(&self, builtin_function: &mut TypedNode) {
        let TypedNode::BuiltinCall { index, arguments, result } = builtin_function else { unreachable!() };
        let kind = self.find_node_kind(&arguments[0]);
        let function = get_builtin(*index);
        if !function.have_instruction || matches!(kind, Kind::String | Kind::Array) { return; }
        match function.name {
            "+" => { *builtin_function = TypedNode::Add(std::mem::take(arguments), kind, *result) },
            "-" => { *builtin_function = TypedNode::Minus(std::mem::take(arguments), kind, *result) },
            "*" => { *builtin_function = TypedNode::Multiply(std::mem::take(arguments), kind, *result) },
            ">" => { *builtin_function = TypedNode::GreaterThan(std::mem::take(arguments), kind, *result) },
            "==" => { *builtin_function = TypedNode::Equal(std::mem::take(arguments), kind, *result) },
            "<" => { *builtin_function = TypedNode::LessThan(std::mem::take(arguments), kind, *result) },
            "<=" => { *builtin_function = TypedNode::LessThanOrEqual(std::mem::take(arguments), kind, *result) },
            ">=" => { *builtin_function = TypedNode::GreaterThanOrEqual(std::mem::take(arguments), kind, *result) },
            "!=" => { *builtin_function = TypedNode::NotEqual(std::mem::take(arguments), kind, *result) },
            _ => { unreachable!() }
        }
    }
}

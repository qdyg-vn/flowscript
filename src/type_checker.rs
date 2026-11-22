use crate::builtins::{get_builtin, Signature};
use crate::error_handler::{Error, ErrorHandler, TypeError, TypeErrorType};
use crate::node::{ResolvedNode, TypedAST, TypedNode, AST, ConditionBranch};
use crate::value::Kind;
use crate::builtins::get_signatures;
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
            match self.check(&station, &typed_nodes, Kind::Undefined) {
                Ok(station) => typed_nodes.push(station),
                Err(errors) => self.error_handler.errors.extend(errors),
            }
        }
        if !self.error_handler.errors.is_empty() { self.error_handler.report_exit() }
        (self.error_handler, TypedAST { nodes: typed_nodes, arity: ast.arity, variables_count: ast.variables_count, max_relative_reference: ast.max_relative_reference })
    }

    pub fn check(&mut self, station: &ResolvedNode, typed_stations: &[TypedNode], expected_return_kind: Kind) -> Result<TypedNode, Vec<Error>> {
        match station {
            ResolvedNode::Literal(value) => Ok(TypedNode::Literal(*value)),
            ResolvedNode::HeavyLiteral(value) => Ok(TypedNode::HeavyLiteral(value.clone())),
            ResolvedNode::BuiltinCall { index, arguments } => {
                let signatures = get_signatures(*index);
                let typed_arguments: Vec<TypedNode> = arguments.iter().map(|argument| self.check(argument, typed_stations, expected_return_kind)).collect::<Result<_, _>>()?;
                let arguments_kind: Vec<Kind> = typed_arguments.iter().map(|argument| self.find_node_kind(argument)).collect();
                match self.find_result_kind(*index, &arguments_kind, signatures) {
                    Ok(result_kind) => {
                        let mut node = TypedNode::BuiltinCall { index: *index, arguments: typed_arguments, result: result_kind };
                        self.rewrite_builtin_function(&mut node);
                        Ok(node)
                    },
                    Err(errors) => Err(errors),
                }
            },
            ResolvedNode::Call { arguments, function_index } => {
                let typed_arguments: Vec<TypedNode> = arguments.iter().map(|argument| self.check(argument, typed_stations, expected_return_kind)).collect::<Result<_, _>>()?;
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
                    let station = self.check(station, &typed_stations, expected_return_kind)?;
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
            ResolvedNode::Assignment(index, variable_index, expected_kind) => {
                let found_kind = self.find_node_kind(typed_stations.last().unwrap());
                if found_kind != *expected_kind { self.error_handler.push_error(TypeError { kind: TypeErrorType::AssignTypeMismatch { found_kind, expected_kind: *expected_kind } }) }
                self.symbol_table.all_variable[*variable_index as usize] = *expected_kind;
                Ok(TypedNode::Assignment(*index, *expected_kind))
            }
            ResolvedNode::DefineFunction { function_index, body, result } => {
                let child_typed_stations = Vec::with_capacity(body.nodes.len());
                Ok(TypedNode::DefineFunction {
                    function_index: *function_index,
                    body: TypedAST {
                        nodes: body.nodes.iter().map(|node| self.check(node, &child_typed_stations, *result)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?,
                        arity: body.arity,
                        variables_count: body.variables_count,
                        max_relative_reference: body.max_relative_reference
                    },
                    result: *result
                })
            },
            ResolvedNode::Condition { branches, final_branch } => {
                let mut typed_branches = Vec::with_capacity(branches.len());
                for branch in branches {
                    typed_branches.push(ConditionBranch { condition: branch.condition.iter().map(|condition| self.check(condition, typed_stations, expected_return_kind)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?, body: branch.body.iter().map(|body| self.check(body, typed_stations, expected_return_kind)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()? })
                };
                Ok(TypedNode::Condition { branches: typed_branches, final_branch: final_branch.iter().map(|body| self.check(body, typed_stations, expected_return_kind)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()? } )
            },
            ResolvedNode::Return(value) => {
                let value = self.check(value, typed_stations, expected_return_kind)?;
                let actual_return_kind = self.find_node_kind(&value);
                if actual_return_kind != expected_return_kind {
                    return Err(vec![TypeError { kind: TypeErrorType::TypeMismatch { expected_kind: expected_return_kind, found_kind: actual_return_kind } }.into()])
                }
                Ok(TypedNode::Return(Box::new(value)))
            },
            ResolvedNode::Array(elements) => Ok(TypedNode::Array(elements.iter().map(|element| self.check(element, typed_stations, expected_return_kind)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?)),
            ResolvedNode::StructInstantiation { index, field_kind_indices, field_values } => {
                let typed_field_values: Vec<TypedNode> = field_values.iter().map(|field_value| self.check(field_value, typed_stations, expected_return_kind)).collect::<Result<_, _>>()?;
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

    fn find_result_kind(&mut self, function_index: u16, arguments_kind: &[Kind], signatures: &[Signature]) -> Result<Kind, Vec<Error>> {
        let mut errors = Vec::with_capacity(arguments_kind.len());
        let mut mismatch_count = usize::MAX;
        for signature in signatures {
            let mut signature_errors = Vec::with_capacity(arguments_kind.len());
            if arguments_kind.len() < signature.fixed.len() {
                let function = get_builtin(function_index);
                signature_errors.push(TypeError{ kind: TypeErrorType::ArityMismatch { function: function.name.to_string(), expected_arity: signature.fixed.len(), found_arity: arguments_kind.len() } }.into());
            } else {
                let (fixed, variadic) = arguments_kind.split_at(signature.fixed.len());
                self.match_fixed_arity_signature(signature.fixed, fixed, &mut signature_errors);
                if let Some(&parameter_kind) = signature.variadic.first() {
                    self.match_variadic_signature(parameter_kind, variadic, &mut signature_errors);
                } else if !variadic.is_empty() {
                    let function = get_builtin(function_index);
                    signature_errors.push(TypeError{ kind: TypeErrorType::ArityMismatch { function: function.name.to_string(), expected_arity: signature.fixed.len(), found_arity: arguments_kind.len() } }.into());
                }
            }
            if signature_errors.is_empty() {
                return Ok(signature.result)
            }
            if signature_errors.len() <= mismatch_count {
                mismatch_count = signature_errors.len();
                errors = signature_errors;
            }
        }
        Err(errors)
    }

    fn match_fixed_arity_signature(&self, parameter_kinds: &[Kind], argument_kinds: &[Kind], signature_errors: &mut Vec<Error>) {
        for (&parameter_kind, &argument_kind) in parameter_kinds.iter().zip(argument_kinds) {
            if parameter_kind != argument_kind {
                signature_errors.push(TypeError { kind: TypeErrorType::TypeMismatch { expected_kind: parameter_kind, found_kind: argument_kind } }.into());
            }
        }
    }

    fn match_variadic_signature(&self, parameter_kind: Kind, argument_kinds: &[Kind], signature_errors: &mut Vec<Error>) {
        for &argument_kind in argument_kinds {
            if parameter_kind != argument_kind {
                signature_errors.push(TypeError { kind: TypeErrorType::TypeMismatch { expected_kind: parameter_kind, found_kind: argument_kind } }.into());
            }
        }
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
        if arguments.is_empty() { return; }
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

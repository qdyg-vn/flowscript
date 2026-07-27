use crate::builtins::Signature;
use crate::error_handler::{Error, ErrorHandler, TypeError, TypeErrorType};
use crate::node::{ResolvedNode, TypedAST, TypedNode, AST};
use crate::value::{ParentKind, VariableType};
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
            match self.check(station, &typed_nodes) {
                Ok(station) => typed_nodes.push(station),
                Err(errors) => self.error_handler.errors.extend(errors),
            }
        }
        if !self.error_handler.errors.is_empty() { self.error_handler.report_exit() }
        (self.error_handler, TypedAST { nodes: typed_nodes, arity: ast.arity, variables_count: ast.variables_count })
    }

    pub fn check(&mut self, station: ResolvedNode, typed_stations: &[TypedNode]) -> Result<TypedNode, Vec<Error>> {
        match station {
            ResolvedNode::Literal(value) => Ok(TypedNode::Literal(value)),
            ResolvedNode::HeavyLiteral(value) => Ok(TypedNode::HeavyLiteral(value.clone())),
            ResolvedNode::BuiltinCall { index, arguments } => {
                let types = get_types(index);
                match self.find_result_parent_kind(&arguments, types, typed_stations) {
                    Ok(result_kind) => Ok(TypedNode::BuiltinCall { index, arguments: arguments.into_iter().map(|argument| self.check(argument, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?, result: result_kind}),
                    Err(errors) => Err(errors),
                }
            },
            ResolvedNode::Call { scope, index, arguments, signature_index } => {
                let typed_arguments = arguments.into_iter().map(|argument| self.check(argument, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?;
                let required_variables_type = self.symbol_table.get_arguments(signature_index);
                self.check_function_arguments(&typed_arguments, required_variables_type);
                Ok(TypedNode::Call { scope, index, arguments: typed_arguments })
            },
            ResolvedNode::Pipeline(stations) => {
                let mut typed_stations = Vec::with_capacity(stations.len());
                for station in stations {
                    typed_stations.push(self.check(station, &typed_stations)?)
                }
                Ok(TypedNode::Pipeline(typed_stations))
            },
            ResolvedNode::RelativeReference(x, y) => {
                if typed_stations.len() < x as usize {
                    return Err(vec![TypeError{ kind: TypeErrorType::MissingStation(x) }.into()])
                }
                let station = &typed_stations[typed_stations.len() - x as usize];
                Ok(TypedNode::RelativeReference(x, y, self.find_typed_node_parent_kind(station)))
            },
            ResolvedNode::Variable(index, kind) => Ok(TypedNode::Variable(index, kind)),
            ResolvedNode::Assignment(index, kind) => Ok(TypedNode::Assignment(index, kind)),
            ResolvedNode::DefineFunction { index, body } => {
                let mut child_typed_stations = Vec::with_capacity(body.nodes.len());
                Ok(TypedNode::DefineFunction { index, body: TypedAST { nodes: body.nodes.into_iter().map(|node| self.check(node, &mut child_typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?, arity: body.arity, variables_count: body.variables_count } })
            },
            ResolvedNode::Condition { branches, final_branch } => {
                let mut typed_branches = Vec::with_capacity(branches.len());
                for branch in branches {
                    typed_branches.push((branch.0.into_iter().map(|condition| self.check(condition, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?, branch.1.into_iter().map(|body| self.check(body, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?))
                };
                Ok(TypedNode::Condition { branches: typed_branches, final_branch: final_branch.into_iter().map(|body| self.check(body, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()? } )
            },
            ResolvedNode::Return(value) => Ok(TypedNode::Return(Box::new(self.check(*value, typed_stations)?))),
            ResolvedNode::Array(elements) => Ok(TypedNode::Array(elements.into_iter().map(|element| self.check(element, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?)),
        }
    }

    fn find_result_parent_kind(&self, arguments: &[ResolvedNode], types: &[Signature], typed_station: &[TypedNode]) -> Result<ParentKind, Vec<Error>> {
        let mut arguments_parent_kind = Vec::with_capacity(arguments.len());
        let mut errors = Vec::with_capacity(arguments.len());
        for argument in arguments {
            match self.find_resolved_node_parent_kind(argument, typed_station) {
                Ok(received_kind) => arguments_parent_kind.push(received_kind),
                Err(error) => errors.extend(error),
            }
        }
        if arguments_parent_kind.len() != arguments.len() { return Err(errors) }
        let mut mismatch_count = i32::MAX;
        for signature in types {
            let mut signature_mismatch_count = 0;
            let mut signature_errors = Vec::with_capacity(arguments.len());
            if signature.infinite_arity {
                if arguments_parent_kind.len() < signature.min_arity as usize {
                    signature_mismatch_count += 1;
                    signature_errors.push(TypeError{ kind: TypeErrorType::ArityMismatch(arguments_parent_kind.len(), signature.min_arity as usize) }.into());
                } else {
                    for argument_kind in &arguments_parent_kind {
                        if *argument_kind != signature.arguments[0] {
                            signature_mismatch_count += 1;
                            signature_errors.push(TypeError { kind: TypeErrorType::ParentKindMismatch(signature.arguments[0], *argument_kind) }.into());
                        }
                    }
                }
            } else {
                if arguments_parent_kind.len() != signature.min_arity as usize {
                    signature_mismatch_count += 1;
                    signature_errors.push(TypeError{ kind: TypeErrorType::ArityMismatch(arguments_parent_kind.len(), signature.min_arity as usize) }.into());
                } else {
                    for index in 0..signature.arguments.len() {
                        let argument_kind = arguments_parent_kind[index];
                        let required_kind = signature.arguments[index];
                        if argument_kind != required_kind {
                            signature_mismatch_count += 1;
                            signature_errors.push(TypeError { kind: TypeErrorType::ParentKindMismatch(required_kind, argument_kind) }.into());
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

    fn find_typed_node_parent_kind(&self, node: &TypedNode) -> ParentKind {
        match node {
            TypedNode::Literal(value) => value.get_parent_kind(),
            TypedNode::HeavyLiteral(value) => value.get_parent_kind(),
            TypedNode::Variable(_, kind) | TypedNode::Assignment(_, kind) => kind.get_parent_kind(),
            TypedNode::BuiltinCall {result, ..} => *result,
            TypedNode::RelativeReference(_, _, parent_kind) => *parent_kind,
            _ => {todo!("Currently under development")}
        }
    }

    fn find_resolved_node_parent_kind(&self, node: &ResolvedNode, typed_station: &[TypedNode]) -> Result<ParentKind, Vec<Error>> {
        match node {
            ResolvedNode::Literal(value) => Ok(value.get_parent_kind()),
            ResolvedNode::HeavyLiteral(value) => Ok(value.get_parent_kind()),
            ResolvedNode::Variable(_, kind) | ResolvedNode::Assignment(_, kind) => Ok(kind.get_parent_kind()),
            ResolvedNode::BuiltinCall { index, arguments } => {
                let types = get_types(*index);
                self.find_result_parent_kind(arguments, types, typed_station)
            },
            ResolvedNode::RelativeReference(x, y) => {
                if typed_station.len() < *x as usize {
                    return Err(vec![TypeError{ kind: TypeErrorType::MissingStation(*x) }.into()]) }
                let station = &typed_station[typed_station.len() - *x as usize];
                Ok(self.find_typed_node_parent_kind(station))
            },
            _ => {todo!("Currently under development")}
        }
    }

    fn check_function_arguments(&mut self, arguments: &[TypedNode], required_variables_type: Vec<VariableType>) {
        let received_arity = arguments.len();
        let required_arity = required_variables_type.len();
        for index in 0..std::cmp::min(received_arity, required_arity) {
            let required_variable_type = required_variables_type[index];
            match &arguments[index] {
                TypedNode::Literal(value) => {
                    let received_variable_type = value.get_kind().get_variable_type();
                    if required_variable_type != VariableType::Dynamic && received_variable_type != VariableType::Dynamic && received_variable_type != required_variable_type {
                        self.error_handler.push_error(TypeError {kind: TypeErrorType::TypeMismatch(required_variable_type.get_kind(), received_variable_type.get_kind())})
                    }
                },
                TypedNode::HeavyLiteral(value) => {
                    let received_variable_type = value.get_kind().get_variable_type();
                    if required_variable_type != VariableType::Dynamic && received_variable_type != VariableType::Dynamic && received_variable_type != required_variable_type {
                        self.error_handler.push_error(TypeError {kind: TypeErrorType::TypeMismatch(required_variable_type.get_kind(), received_variable_type.get_kind())})
                    }
                },
                TypedNode::Variable(_, kind) => {
                    let received_variable_type = kind.get_variable_type();
                    if required_variable_type != VariableType::Dynamic && received_variable_type != VariableType::Dynamic && received_variable_type != required_variable_type {
                        self.error_handler.push_error(TypeError {kind: TypeErrorType::TypeMismatch(required_variable_type.get_kind(), received_variable_type.get_kind())})
                    }
                },
                TypedNode::RelativeReference(_, _, parent_kind) => {
                    if required_variable_type != VariableType::Dynamic && required_variable_type.get_parent_kind() != *parent_kind {
                        self.error_handler.push_error(TypeError {kind: TypeErrorType::ParentKindMismatch(required_variable_type.get_parent_kind(), *parent_kind)})
                    }
                },
                TypedNode::BuiltinCall { result, .. } => {
                    if required_variable_type != VariableType::Dynamic && required_variable_type.get_parent_kind() != *result {
                        self.error_handler.push_error(TypeError {kind: TypeErrorType::ParentKindMismatch(required_variable_type.get_parent_kind(), *result)})
                    }
                }
                _ => {todo!("Currently under development")}
            }
        }
        if received_arity != required_arity {
            self.error_handler.push_error(TypeError {kind: TypeErrorType::ArityMismatch(received_arity, required_arity)})
        }
    }
}

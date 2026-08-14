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
                match self.find_result_kind(&arguments, types, typed_stations) {
                    Ok(result_kind) => {
                        let mut node = TypedNode::BuiltinCall { index, arguments: arguments.into_iter().map(|argument| self.check(argument, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?, result: result_kind };
                        self.rewrite_builtin_function(&mut node);
                        Ok(node)
                    },
                    Err(errors) => Err(errors),
                }
            },
            ResolvedNode::Call { arguments, signature_index } => {
                let typed_arguments = arguments.into_iter().map(|argument| self.check(argument, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?;
                let required_variables_type = self.symbol_table.get_parameters(signature_index);
                self.check_function_arguments(&typed_arguments, required_variables_type);
                Ok(TypedNode::Call { signature_index, arguments: typed_arguments, result: self.symbol_table.get_result(signature_index) })
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
                Ok(TypedNode::RelativeReference(x, y, self.find_typed_node_kind(station)))
            },
            ResolvedNode::Variable(index, variable_index) => {
                let kind = self.symbol_table.all_variable[variable_index as usize];
                Ok(TypedNode::Variable(index, kind))
            },
            ResolvedNode::SoftAssignment(index, variable_index) => {
                let received_kind = match typed_stations.last() {
                    Some(typed_node) => self.find_typed_node_kind(typed_node),
                    None => { unreachable!() }
                };
                self.symbol_table.all_variable[variable_index as usize] = received_kind;
                Ok(TypedNode::Assignment(index, received_kind))
            },
            ResolvedNode::Assignment(index, variable_index, kind) => {
                let received_kind = match typed_stations.last() {
                    Some(typed_node) => self.find_typed_node_kind(typed_node),
                    None => { unreachable!() }
                };
                if received_kind != kind { self.error_handler.push_error(TypeError { kind: TypeErrorType::AssignTypeMismatch(received_kind, kind) }) }
                self.symbol_table.all_variable[variable_index as usize] = kind;
                Ok(TypedNode::Assignment(index, kind))
            },
            ResolvedNode::DefineFunction { signature_index, body } => {
                let child_typed_stations = Vec::with_capacity(body.nodes.len());
                Ok(TypedNode::DefineFunction { signature_index, body: TypedAST { nodes: body.nodes.into_iter().map(|node| self.check(node, &child_typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?, arity: body.arity, variables_count: body.variables_count }})
            },
            ResolvedNode::Condition { branches, final_branch } => {
                let mut typed_branches = Vec::with_capacity(branches.len());
                for branch in branches {
                    typed_branches.push(ConditionBranch { condition: branch.condition.into_iter().map(|condition| self.check(condition, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?, body: branch.body.into_iter().map(|body| self.check(body, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()? })
                };
                Ok(TypedNode::Condition { branches: typed_branches, final_branch: final_branch.into_iter().map(|body| self.check(body, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()? } )
            },
            ResolvedNode::Return(value) => Ok(TypedNode::Return(Box::new(self.check(*value, typed_stations)?))),
            ResolvedNode::Array(elements) => Ok(TypedNode::Array(elements.into_iter().map(|element| self.check(element, typed_stations)).collect::<Result<Vec<TypedNode>, Vec<Error>>>()?)),
        }
    }

    fn find_result_kind(&self, arguments: &[ResolvedNode], types: &[Signature], typed_station: &[TypedNode]) -> Result<Kind, Vec<Error>> {
        let mut arguments_parent_kind = Vec::with_capacity(arguments.len());
        let mut errors = Vec::with_capacity(arguments.len());
        for argument in arguments {
            match self.find_resolved_node_kind(argument, typed_station) {
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
                            signature_errors.push(TypeError { kind: TypeErrorType::TypeMismatch(signature.arguments[0], *argument_kind) }.into());
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

    fn find_typed_node_kind(&self, node: &TypedNode) -> Kind {
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
            _ => {todo!("Currently under development")}
        }
    }

    fn find_resolved_node_kind(&self, node: &ResolvedNode, typed_station: &[TypedNode]) -> Result<Kind, Vec<Error>> {
        match node {
            ResolvedNode::Literal(value) => Ok(value.get_kind()),
            ResolvedNode::HeavyLiteral(value) => Ok(value.get_kind()),
            ResolvedNode::Variable(_, variable_index) | ResolvedNode::SoftAssignment(_, variable_index) => {
                let kind = self.symbol_table.all_variable[*variable_index as usize];
                Ok(kind)
            },
            ResolvedNode::Assignment(_, _, kind) => Ok(*kind),
            ResolvedNode::BuiltinCall { index, arguments } => {
                let types = get_types(*index);
                self.find_result_kind(arguments, types, typed_station)
            },
            ResolvedNode::RelativeReference(x, y) => {
                if typed_station.len() < *x as usize {
                    return Err(vec![TypeError{ kind: TypeErrorType::MissingStation(*x) }.into()]) }
                let station = &typed_station[typed_station.len() - *x as usize];
                Ok(self.find_typed_node_kind(station))
            },
            _ => {todo!("Currently under development")}
        }
    }

    fn check_function_arguments(&mut self, arguments: &[TypedNode], required_variables_type: Vec<Kind>) {
        let received_arity = arguments.len();
        let required_arity = required_variables_type.len();
        for index in 0..std::cmp::min(received_arity, required_arity) {
            let required_variable_type = required_variables_type[index];
            match &arguments[index] {
                TypedNode::Literal(value) => {
                    let received_variable_type = value.get_kind();
                    if received_variable_type != required_variable_type {
                        self.error_handler.push_error(TypeError {kind: TypeErrorType::TypeMismatch(required_variable_type, received_variable_type)})
                    }
                },
                TypedNode::HeavyLiteral(value) => {
                    let received_variable_type = value.get_kind();
                    if received_variable_type != required_variable_type {
                        self.error_handler.push_error(TypeError {kind: TypeErrorType::TypeMismatch(required_variable_type, received_variable_type)})
                    }
                },
                TypedNode::Variable(_, kind) => {
                    if *kind != required_variable_type {
                        self.error_handler.push_error(TypeError {kind: TypeErrorType::TypeMismatch(required_variable_type, *kind)})
                    }
                },
                TypedNode::RelativeReference(_, _, kind) => {
                    if required_variable_type != *kind {
                        self.error_handler.push_error(TypeError {kind: TypeErrorType::TypeMismatch(required_variable_type, *kind)})
                    }
                },
                TypedNode::BuiltinCall { result, .. } | TypedNode::Add(_, _, result)
                | TypedNode::Minus(_, _, result) | TypedNode::Multiply(_, _, result)
                | TypedNode::Equal(_, _, result) | TypedNode::LessThan(_, _, result)
                | TypedNode::GreaterThan(_, _, result) | TypedNode::LessThanOrEqual(_, _, result)
                | TypedNode::GreaterThanOrEqual(_, _, result) | TypedNode::NotEqual(_, _, result)
                => {
                    if required_variable_type != *result {
                        self.error_handler.push_error(TypeError {kind: TypeErrorType::TypeMismatch(required_variable_type, *result)})
                    }
                },
                _ => {todo!("Currently under development")}
            }
        }
        if received_arity != required_arity {
            self.error_handler.push_error(TypeError {kind: TypeErrorType::ArityMismatch(received_arity, required_arity)})
        }
    }

    fn rewrite_builtin_function(&self, builtin_function: &mut TypedNode) {
        let TypedNode::BuiltinCall { index, arguments, result } = builtin_function else { unreachable!() };
        let kind = self.find_typed_node_kind(&arguments[0]);
        let function = get_builtin(*index);
        if !function.have_instruction { return; }
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

use crate::error_handler::{Error, ErrorHandler, SemanticError, SemanticErrorType, TypeError, TypeErrorType};
use crate::node::{Node, ResolvedNode, AST};
use crate::symbol_table::{SymbolTable, SymbolType};
use crate::value::VariableType;

pub struct Resolver {
    error_handler: ErrorHandler,
    symbol_table: SymbolTable,
}
impl Resolver {
    pub fn new(error_handler: ErrorHandler, symbol_table: SymbolTable) -> Self {
        Self { error_handler, symbol_table }
    }

    pub fn resolve(mut self, nodes: Vec<Node>) -> (ErrorHandler, usize, AST) {
        let mut ast = AST {nodes: Vec::with_capacity(nodes.len()), arity: 0};
        for node in nodes {
            if let Node::Pipeline(stations) = node {
                let result = ResolvedNode::Pipeline(self.solve(stations, &mut ast));
                ast.nodes.push(result)
            }
        }
        if !self.error_handler.errors.is_empty() { self.error_handler.report_exit() }
        (self.error_handler, ast.arity as usize, ast)
    }

    fn solve(&mut self, stations: Vec<Node>, ast: &mut AST) -> Vec<ResolvedNode> {
        let mut resolved_stations = Vec::with_capacity(stations.len());
        for station in stations {
            match station {
                Node::Literal(value) => resolved_stations.push(ResolvedNode::Literal(value)),
                Node::HeavyLiteral(value) => resolved_stations.push(ResolvedNode::HeavyLiteral(value)),
                Node::RelativeReference(x, y) => resolved_stations.push(ResolvedNode::RelativeReference(x, y)),
                Node::Apply {operator, arguments} => {
                    match self.symbol_table.resolve(&operator) {
                        Ok(SymbolType::Builtin(index)) => resolved_stations.push(ResolvedNode::BuiltinCall {index, arguments: self.solve(arguments, ast)}),
                        Ok(SymbolType::Scope(scope, index, function)) => {
                            let arguments = self.solve(arguments, ast);
                            if let VariableType::Function(function_index) = function {
                                let required_variables_type = self.symbol_table.get_arguments(function_index);
                                self.check_function_arguments(&arguments, required_variables_type)
                            }
                            resolved_stations.push(ResolvedNode::Call { scope, index, arguments })
                        },
                        Err(error) => self.error_handler.errors.push(Error::SemanticError(error))
                    }
                },
                Node::SoftAssignment(name) => {
                    let kind = match resolved_stations.last() {
                        Some(ResolvedNode::Literal(value)) => {
                            value.get_kind()
                        },
                        Some(ResolvedNode::HeavyLiteral(value)) => {
                            value.get_kind()
                        },
                        Some(ResolvedNode::Variable(_, kind)) | Some(ResolvedNode::Assignment(_, kind)) => *kind,
                        _ => {todo!()}
                    };
                    match self.symbol_table.add_variable(name, kind.get_variable_type()) {
                        Ok(SymbolType::Scope(_, index, _)) => {
                            resolved_stations.push(ResolvedNode::Assignment(index, kind));
                            ast.arity += 1
                        }
                        Err(SymbolType::Scope(_, index, variable_type)) => {
                            if kind.get_variable_type() != variable_type {
                                self.error_handler.errors.push(Error::TypeError(TypeError { kind: TypeErrorType::AssignTypeMismatch(kind, variable_type.get_kind()) }))
                            }
                            resolved_stations.push(ResolvedNode::Assignment(index, kind))
                        },
                        _ => unreachable!()
                    }
                },
                Node::Assignment(name, kind) => {
                    match resolved_stations.last() {
                        Some(ResolvedNode::Literal(value)) => {
                            let received_kind = value.get_kind();
                            if received_kind != kind { self.error_handler.errors.push(Error::TypeError(TypeError { kind: TypeErrorType::AssignTypeMismatch(received_kind, kind) })) }
                        },
                        Some(ResolvedNode::HeavyLiteral(value)) => {
                            let received_kind = value.get_kind();
                            if received_kind != kind { self.error_handler.errors.push(Error::TypeError(TypeError {kind: TypeErrorType::AssignTypeMismatch(received_kind, kind) })) }
                        },
                        _ => {}
                    };
                    match self.symbol_table.add_variable(name, kind.get_variable_type()) {
                        Ok(SymbolType::Scope(_, index, _)) => {
                            resolved_stations.push(ResolvedNode::Assignment(index, kind));
                            ast.arity += 1
                        }
                        Err(SymbolType::Scope(_, index, _)) => resolved_stations.push(ResolvedNode::Assignment(index, kind)),
                        _ => unreachable!()
                    }
                },
                Node::Variable(name) => {
                    match self.symbol_table.resolve(&name) {
                        Ok(SymbolType::Scope(_, index, variable_type)) => resolved_stations.push(ResolvedNode::Variable(index, variable_type.get_kind())),
                        Err(error) => self.error_handler.errors.push(Error::SemanticError(error)),
                        _ => unreachable!()
                    }
                },
                Node::DefineFunction {operator, arguments, body} => {
                    let index = match self.symbol_table.add_variable(operator, VariableType::Function(arguments.len() as u32)) {
                        Ok(SymbolType::Scope(_, index, _)) => {
                            ast.arity += 1;
                            index
                        },
                        Err(SymbolType::Scope(_, index, _)) => {
                            index
                        },
                        _ => unreachable!()
                    };
                    self.symbol_table.new_scope();
                    let mut child_ast = AST {nodes: Vec::with_capacity(arguments.len() + body.len()), arity: 0};
                    for argument in arguments {
                        match argument {
                            Node::SoftAssignment(name) => {
                                match self.symbol_table.add_variable(name.clone(), VariableType::Dynamic) {
                                    Ok(SymbolType::Scope(_, _, _)) => {
                                        child_ast.arity += 1;
                                        self.symbol_table.all_arguments.push(VariableType::Dynamic)
                                    },
                                    Err(SymbolType::Scope(_, _, _)) => self.error_handler.errors.push(Error::SemanticError(SemanticError {kind: SemanticErrorType::DuplicateParameter(name)})),
                                    _ => unreachable!()
                                }
                            }
                            Node::Assignment(name, kind) => {
                                match self.symbol_table.add_variable(name.clone(), kind.get_variable_type()) {
                                    Ok(SymbolType::Scope(_, _, _)) => {
                                        child_ast.arity += 1;
                                        self.symbol_table.all_arguments.push(kind.get_variable_type())
                                    },
                                    Err(SymbolType::Scope(_, _, _)) => self.error_handler.errors.push(Error::SemanticError(SemanticError {kind: SemanticErrorType::DuplicateParameter(name)})),
                                    _ => unreachable!()
                                }
                            }
                            _ => todo!()
                        }
                    }
                    let body = self.solve(body, &mut child_ast);
                    child_ast.nodes.extend(body);
                    self.symbol_table.scopes.pop();
                    resolved_stations.push(ResolvedNode::DefineFunction {index, body: child_ast})
                },
                Node::Pipeline(stations) => {
                    let stations = self.solve(stations, ast);
                    resolved_stations.push(ResolvedNode::Pipeline(stations))
                },
                Node::Condition {branches, final_branch} => {
                    let mut resolved_branches = Vec::with_capacity(branches.len());
                    for branch in branches {
                        resolved_branches.push((self.solve(branch.0, ast), self.solve(branch.1, ast)))
                    }
                    let final_branch = self.solve(final_branch, ast);
                    resolved_stations.push(ResolvedNode::Condition {branches: resolved_branches, final_branch})
                },
                Node::Return(value) => {
                    let value = self.solve(vec![*value], ast).pop().unwrap();
                    resolved_stations.push(ResolvedNode::Return(Box::new(value)))
                },
                Node::Array(elements) => {
                    let elements = self.solve(elements, ast);
                    resolved_stations.push(ResolvedNode::Array(elements))
                }
            }
        }
        resolved_stations
    }

    fn check_function_arguments(&mut self, arguments: &[ResolvedNode], required_variables_type: Vec<VariableType>) {
        let received_arity = arguments.len();
        let required_arity = required_variables_type.len();
        for index in 0..std::cmp::min(received_arity, required_arity) {
            let required_variable_type = required_variables_type[index];
            match &arguments[index] {
                ResolvedNode::Literal(value) => {
                    let received_variable_type = value.get_kind().get_variable_type();
                    if required_variable_type != VariableType::Dynamic && received_variable_type != VariableType::Dynamic && received_variable_type != required_variable_type {
                        self.error_handler.errors.push(Error::TypeError(TypeError {kind: TypeErrorType::TypeMismatch(required_variable_type.get_kind(), received_variable_type.get_kind())}))
                    }
                },
                ResolvedNode::HeavyLiteral(value) => {
                    let received_variable_type = value.get_kind().get_variable_type();
                    if required_variable_type != VariableType::Dynamic && received_variable_type != VariableType::Dynamic && received_variable_type != required_variable_type {
                        self.error_handler.errors.push(Error::TypeError(TypeError {kind: TypeErrorType::TypeMismatch(required_variable_type.get_kind(), received_variable_type.get_kind())}))
                    }
                },
                ResolvedNode::Variable(_, kind) => {
                    let received_variable_type = kind.get_variable_type();
                    if required_variable_type != VariableType::Dynamic && received_variable_type != VariableType::Dynamic && received_variable_type != required_variable_type {
                        self.error_handler.errors.push(Error::TypeError(TypeError {kind: TypeErrorType::TypeMismatch(required_variable_type.get_kind(), received_variable_type.get_kind())}))
                    }
                }
                _ => {todo!("Currently under development")}
            }
        }
        if received_arity != required_arity {
            self.error_handler.errors.push(Error::TypeError(TypeError {kind: TypeErrorType::ArityMismatchError(received_arity, required_arity)}))
        }
    }
}
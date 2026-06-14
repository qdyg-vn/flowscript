use crate::error_handler::{Error, ErrorHandler, SemanticError, SemanticErrorType, TypeError, TypeErrorType};
use crate::node::{Node, ResolvedNode, AST};
use crate::symbol_table::{SymbolTable, SymbolType};
use crate::value::{HeavyValue, Kind, LightValue};

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
                        Ok(SymbolType::Scope(scope, index)) => resolved_stations.push(ResolvedNode::Call {scope, index, arguments: self.solve(arguments, ast)}),
                        Err(error) => self.error_handler.errors.push(Error::SemanticError(error))
                    }
                },
                Node::Assignment(name) => {
                    match self.symbol_table.add_variable(name) {
                        Ok(SymbolType::Scope(_, index)) => {
                            resolved_stations.push(ResolvedNode::Assignment(index));
                            ast.arity += 1
                        }
                        Err(SymbolType::Scope(_, index)) => resolved_stations.push(ResolvedNode::Assignment(index)),
                        _ => unreachable!()
                    }
                },
                Node::HardAssignment(name, kind) => {
                    match resolved_stations.last() {
                        Some(ResolvedNode::Literal(value)) => {
                            let received_kind = self.get_type(*value);
                            if received_kind != kind { self.error_handler.errors.push(Error::TypeError(TypeError { kind: TypeErrorType::AssignTypeMismatch(received_kind, kind) })) }
                        },
                        Some(ResolvedNode::HeavyLiteral(value)) => {
                            let received_kind = self.get_heavy_type(value);
                            if received_kind != kind { self.error_handler.errors.push(Error::TypeError(TypeError {kind: TypeErrorType::AssignTypeMismatch(received_kind, kind) })) }
                        },
                        _ => {}
                    };
                    match self.symbol_table.add_variable(name) {
                        Ok(SymbolType::Scope(_, index)) => {
                            resolved_stations.push(ResolvedNode::HardAssignment(index, kind));
                            ast.arity += 1
                        }
                        Err(SymbolType::Scope(_, index)) => resolved_stations.push(ResolvedNode::HardAssignment(index, kind)),
                        _ => unreachable!()
                    }
                },
                Node::Variable(name) => {
                    match self.symbol_table.resolve(&name) {
                        Ok(SymbolType::Scope(_, index)) => resolved_stations.push(ResolvedNode::Variable(index)),
                        Err(error) => self.error_handler.errors.push(Error::SemanticError(error)),
                        _ => unreachable!()
                    }
                },
                Node::DefineFunction {operator, arguments, body} => {
                    let index = match self.symbol_table.add_variable(operator) {
                        Ok(SymbolType::Scope(_, index)) => {
                            ast.arity += 1;
                            index
                        },
                        Err(SymbolType::Scope(_, index)) => {
                            index
                        },
                        _ => unreachable!()
                    };
                    self.symbol_table.new_scope();
                    let mut child_ast = AST {nodes: Vec::with_capacity(arguments.len() + body.len()), arity: 0};
                    for argument in arguments {
                        match argument {
                            Node::Assignment(name) => {
                                match self.symbol_table.add_variable(name.clone()) {
                                    Ok(_) => child_ast.arity += 1,
                                    Err(SymbolType::Scope(_, _)) => self.error_handler.errors.push(Error::SemanticError(SemanticError {kind: SemanticErrorType::DuplicateParameter(name)})),
                                    _ => unreachable!()
                                }
                            }
                            Node::HardAssignment(name, kind) => {
                                match self.symbol_table.add_variable(name.clone()) {
                                    Ok(SymbolType::Scope(_, index)) => {
                                        child_ast.nodes.push(ResolvedNode::HardAssignment(index, kind));
                                        child_ast.arity += 1
                                    },
                                    Err(SymbolType::Scope(_, _)) => self.error_handler.errors.push(Error::SemanticError(SemanticError {kind: SemanticErrorType::DuplicateParameter(name)})),
                                    _ => unreachable!()
                                }
                            }
                            _ => todo!()
                        }
                    }
                    child_ast.nodes = self.solve(body, &mut child_ast);
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

    fn get_type(&self, value: LightValue) -> Kind {
        match value {
            LightValue::Boolean(_) => Kind::Boolean,
            LightValue::Integer(_) => Kind::Integer,
            LightValue::Float(_) => Kind::Float,
            _ => todo!()
        }
    }

    fn get_heavy_type(&self, value: &HeavyValue) -> Kind {
        match value {
            HeavyValue::String(_) => Kind::String,
            _ => todo!()
        }
    }
}
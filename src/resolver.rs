use crate::error_handler::{ErrorHandler, SemanticError, SemanticErrorType, TypeError, TypeErrorType};
use crate::node::{Node, ResolvedNode, AST};
use crate::symbol_table::{SymbolTable, SymbolType};
use crate::value::Kind;

pub struct ResolverOutput {
    pub error_handler: ErrorHandler,
    pub symbol_table: SymbolTable,
    pub ast: AST,
    pub total_define_function_count: u32,
    pub main_define_function_count: u32,
    pub main_variables_count: u16,
}

pub struct Resolver {
    error_handler: ErrorHandler,
    symbol_table: SymbolTable,
    total_define_function_count: u32,
}
impl Resolver {
    pub fn new(error_handler: ErrorHandler, symbol_table: SymbolTable) -> Self {
        Self { error_handler, symbol_table, total_define_function_count: 0 }
    }

    pub fn resolve(mut self, nodes: Vec<Node>) -> ResolverOutput {
        let mut ast = AST { nodes: Vec::with_capacity(nodes.len()), arity: 0, variables_count: 0 };
        let main_define_function_count = self.push_signature_to_symbol_table(&nodes, &mut ast);
        for node in nodes {
            if let Node::Pipeline(stations) = node {
                let result = ResolvedNode::Pipeline(self.solve(stations, &mut ast));
                ast.nodes.push(result)
            }
        }
        if !self.error_handler.errors.is_empty() { self.error_handler.report_exit() }
        ResolverOutput { error_handler: self.error_handler, symbol_table: self.symbol_table, main_variables_count: ast.variables_count, ast, total_define_function_count: self.total_define_function_count, main_define_function_count }
    }

    fn push_signature_to_symbol_table(&mut self, stations: &Vec<Node>, ast: &mut AST) -> u32 {
        let mut define_function_count = 0;
        for station in stations {
            match station {
                Node::Pipeline(stations) => { self.push_signature_to_symbol_table(stations, ast); },
                Node::DefineFunction { operator, parameters, result, .. } => {
                    self.symbol_table.add_function(operator.clone(), parameters.len() as u32, *result);
                    define_function_count += 1;
                }
                Node::Condition { branches, final_branch } => {
                    for branch in branches {
                        { self.push_signature_to_symbol_table(&branch.1, ast); }
                    }
                    { self.push_signature_to_symbol_table(final_branch, ast); }
                }
                _ => {}
            }
        }
        define_function_count
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
                        Ok(SymbolType::Builtin(index)) => resolved_stations.push(ResolvedNode::BuiltinCall { index, arguments: self.solve(arguments, ast) }),
                        Ok(SymbolType::FunctionScope(signature_index)) => {
                            let arguments = self.solve(arguments, ast);
                            resolved_stations.push(ResolvedNode::Call { arguments, signature_index })
                        },
                        Ok(SymbolType::VariableScope(_, _)) => self.error_handler.push_error(TypeError { kind: TypeErrorType::NotAFunction(operator) }),
                        Err(error) => self.error_handler.push_error(error)
                    }
                },
                Node::SoftAssignment(name) => {
                    match self.symbol_table.add_variable(name, Kind::Undefined) {
                        Ok(SymbolType::VariableScope(index, variable_index)) => {
                            resolved_stations.push(ResolvedNode::SoftAssignment(index, variable_index));
                            ast.variables_count += 1
                        }
                        Err(SymbolType::VariableScope(index, variable_index)) => resolved_stations.push(ResolvedNode::SoftAssignment(index, variable_index)),
                        _ => unreachable!()
                    }
                },
                Node::Assignment(name, kind) => {
                    match self.symbol_table.add_variable(name, kind) {
                        Ok(SymbolType::VariableScope(index, variable_index)) => {
                            resolved_stations.push(ResolvedNode::Assignment(index, variable_index, kind));
                            ast.variables_count += 1
                        }
                        Err(SymbolType::VariableScope(index, variable_index)) => resolved_stations.push(ResolvedNode::Assignment(index, variable_index, kind)),
                        _ => unreachable!()
                    }
                }
                Node::Variable(name) => {
                    match self.symbol_table.resolve(&name) {
                        Ok(SymbolType::VariableScope(index, variable_index)) => resolved_stations.push(ResolvedNode::Variable(index, variable_index)),
                        Err(error) => self.error_handler.push_error(error),
                        _ => unreachable!()
                    }
                },
                Node::DefineFunction { operator, parameters, body, result } => {
                    let signature_index = match self.symbol_table.add_function(operator, parameters.len() as u32, result) {
                        Err(SymbolType::FunctionScope(signature_index)) => signature_index,
                        _ => unreachable!()
                    };
                    self.total_define_function_count += 1;
                    self.symbol_table.new_scope();
                    let mut child_ast = AST { nodes: Vec::with_capacity(body.len()), arity: 0, variables_count: 0 };
                    for parameter in parameters {
                        match parameter {
                            Node::Assignment(name, kind) => {
                                match self.symbol_table.add_variable(name.clone(), kind) {
                                    Ok(SymbolType::VariableScope(_, _)) => {
                                        child_ast.arity += 1;
                                        self.symbol_table.all_parameters.push(kind)
                                    },
                                    Err(SymbolType::VariableScope(_, _)) => self.error_handler.push_error(SemanticError {kind: SemanticErrorType::DuplicateParameter(name)}),
                                    _ => unreachable!()
                                }
                            }
                            _ => todo!()
                        }
                    }
                    self.push_signature_to_symbol_table(&body, &mut child_ast);
                    let body = self.solve(body, &mut child_ast);
                    child_ast.nodes = body;
                    self.symbol_table.pop_scope();
                    resolved_stations.push(ResolvedNode::DefineFunction { signature_index, body: child_ast })
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
}

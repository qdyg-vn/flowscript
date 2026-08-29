use crate::error_handler::{ErrorHandler, SemanticError, SemanticErrorType, TypeError, TypeErrorType};
use crate::node::{Node, ResolvedNode, AST, ConditionBranch};
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

#[derive(Debug, Copy, Clone)]
enum PipelineContext {
    NotInPipeline,
    InPipeline {
        station_index: usize,
    },
    InStation {
        station_index: usize,
    },
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
        let mut ast = AST { nodes: Vec::with_capacity(nodes.len()), .. AST::default() };
        let main_define_function_count = self.push_signature_to_symbol_table(&nodes);
        ast.nodes = self.solve(nodes, &mut ast, PipelineContext::NotInPipeline);
        if !self.error_handler.errors.is_empty() { self.error_handler.report_exit() }
        ResolverOutput { error_handler: self.error_handler, symbol_table: self.symbol_table, main_variables_count: ast.variables_count, ast, total_define_function_count: self.total_define_function_count, main_define_function_count }
    }

    fn push_signature_to_symbol_table(&mut self, stations: &Vec<Node>) -> u32 {
        let mut define_function_count = 0;
        for station in stations {
            match station {
                Node::Pipeline(stations) => { self.push_signature_to_symbol_table(stations); },
                Node::DefineFunction { operator, parameters, result, .. } => {
                    let mut parameters_kind = Vec::with_capacity(parameters.len());
                    for parameter in parameters {
                        let Node::Assignment(_, kind) = parameter else { todo!() };
                        parameters_kind.push(*kind);
                    }
                    if let Some(error) = self.symbol_table.add_function(operator.clone(), parameters_kind, *result) {
                        self.error_handler.push_error(error);
                    }
                    define_function_count += 1;
                }
                Node::Condition { branches, final_branch } => {
                    for branch in branches {
                        { self.push_signature_to_symbol_table(&branch.body); }
                    }
                    { self.push_signature_to_symbol_table(final_branch); }
                }
                _ => {}
            }
        }
        define_function_count
    }

    fn solve(&mut self, stations: Vec<Node>, ast: &mut AST, pipeline_context: PipelineContext) -> Vec<ResolvedNode> {
        let mut resolved_stations = Vec::with_capacity(stations.len());
        for (index, station) in stations.into_iter().enumerate() {
            self.solve_node(station, ast, &mut resolved_stations, if matches!(pipeline_context, PipelineContext::InPipeline { .. }) { PipelineContext::InPipeline { station_index: index } } else { pipeline_context })
        }
        resolved_stations
    }

    fn solve_node(&mut self, station: Node, ast: &mut AST, resolved_stations: &mut Vec<ResolvedNode>, pipeline_context: PipelineContext) {
        match station {
            Node::Literal(value) => resolved_stations.push(ResolvedNode::Literal(value)),
            Node::HeavyLiteral(value) => resolved_stations.push(ResolvedNode::HeavyLiteral(value)),
            Node::RelativeReference(x, y) => {
                let station_index = match pipeline_context {
                    PipelineContext::NotInPipeline => {
                        self.error_handler.push_error(SemanticError {kind: SemanticErrorType::RelativeReferenceNotInPipeline});
                        return;
                    },
                    PipelineContext::InPipeline { station_index } => station_index,
                    PipelineContext::InStation { station_index } => station_index,
                };
                if station_index < x as usize {
                    self.error_handler.push_error(SemanticError {kind: SemanticErrorType::MissingStation(station_index as u16)});
                    return;
                }
                let index_in_stations = station_index as u16 - x;
                let SymbolType::VariableScope(index, variable_index) = self.symbol_table.add_relative_reference(index_in_stations, Kind::Undefined) else { unreachable!() };
                resolved_stations.push(ResolvedNode::RelativeReference(index, y, variable_index))
            },
            Node::Apply {operator, arguments} => {
                let station_context = match pipeline_context {
                    PipelineContext::InPipeline { station_index } => PipelineContext::InStation { station_index },
                    anything_else => anything_else
                };
                match self.symbol_table.resolve(&operator) {
                    Ok(SymbolType::Builtin(index)) => resolved_stations.push(ResolvedNode::BuiltinCall { index, arguments: self.solve(arguments, ast, station_context) }),
                    Ok(SymbolType::FunctionScope(function_index)) => {
                        let arguments = self.solve(arguments, ast, station_context);
                        resolved_stations.push(ResolvedNode::Call { arguments, function_index })
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
            Node::DefineFunction { operator, parameters, body, .. } => {
                let mut child_ast = AST { nodes: Vec::with_capacity(body.len()), .. AST::default() };
                self.total_define_function_count += 1;
                self.symbol_table.new_scope();
                let mut parameters_kind = Vec::with_capacity(parameters.len());
                for parameter in parameters {
                    match parameter {
                        Node::Assignment(name, kind) => {
                            match self.symbol_table.add_variable(name.clone(), kind) {
                                Ok(SymbolType::VariableScope(_, _)) => {
                                    child_ast.arity += 1;
                                    parameters_kind.push(kind);
                                },
                                Err(SymbolType::VariableScope(_, _)) => self.error_handler.push_error(SemanticError {kind: SemanticErrorType::DuplicateParameter(name)}),
                                _ => unreachable!()
                            }
                        }
                        _ => todo!()
                    }
                }
                self.push_signature_to_symbol_table(&body);
                let body = self.solve(body, &mut child_ast, PipelineContext::NotInPipeline);
                child_ast.nodes = body;
                self.symbol_table.pop_scope();
                let Ok(SymbolType::FunctionScope(function_id)) = self.symbol_table.resolve(&operator) else { unreachable!() };
                let function = self.symbol_table.find_function(function_id, parameters_kind).unwrap();
                resolved_stations.push(ResolvedNode::DefineFunction { function_index: function.index, body: child_ast })
            },
            Node::Pipeline(stations) => {
                self.symbol_table.pipeline.clear();
                let mut stations = self.solve(stations, ast, PipelineContext::InPipeline { station_index: 0 });
                self.solve_relative_reference(&mut stations);
                ast.max_relative_reference = std::cmp::max(ast.max_relative_reference, self.symbol_table.pipeline.len() as u8);
                resolved_stations.push(ResolvedNode::Pipeline(stations))
            },
            Node::Condition {branches, final_branch} => {
                let mut resolved_branches = Vec::with_capacity(branches.len());
                for branch in branches {
                    resolved_branches.push(ConditionBranch {
                        condition: self.solve(branch.condition, ast, pipeline_context),
                        body: self.solve(branch.body, ast, pipeline_context)
                    })
                }
                let final_branch = self.solve(final_branch, ast, pipeline_context);
                resolved_stations.push(ResolvedNode::Condition {branches: resolved_branches, final_branch})
            },
            Node::Return(value) => {
                let value = self.solve(vec![*value], ast, pipeline_context).pop().unwrap();
                resolved_stations.push(ResolvedNode::Return(Box::new(value)))
            },
            Node::Array(elements) => {
                let elements = self.solve(elements, ast, pipeline_context);
                resolved_stations.push(ResolvedNode::Array(elements))
            }
        }
    }

    fn solve_relative_reference(&self, stations: &mut Vec<ResolvedNode>) {
        for (index, relative_reference) in self.symbol_table.pipeline.iter().enumerate().rev() {
            stations.insert(relative_reference.index_in_stations as usize + 1, ResolvedNode::StationCapture(index as u16, relative_reference.variable_index));
        }
    }
}

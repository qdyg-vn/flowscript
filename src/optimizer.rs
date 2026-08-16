use crate::node::{ResolvedNode, AST};
use crate::builtins::{get_builtin, BuiltinFunction};
use crate::value::{LightValue, Value};
use crate::error_handler::ErrorHandler;

pub struct Optimizer {
    error_handler: ErrorHandler,
}

impl Optimizer {
    pub fn new(error_handler: ErrorHandler) -> Self {
        Self { error_handler }
    }

    pub fn optimizer(mut self, mut ast: AST) -> (AST, ErrorHandler) {
        self.optimize(&mut ast.nodes);
        (ast, self.error_handler)
    }

    fn optimize(&mut self, stations: &mut Vec<ResolvedNode>) {
        for station in stations {
            self.optimize_node(station)
        }
    }

    fn optimize_node(&mut self, station: &mut ResolvedNode) {
        match station {
            ResolvedNode::BuiltinCall { .. } => self.builtin_node(station),
            ResolvedNode::DefineFunction { body: AST { nodes, .. }, .. } => self.optimize(nodes),
            ResolvedNode::Call { arguments, .. } => self.optimize(arguments),
            ResolvedNode::Pipeline(station) => self.optimize(station),
            ResolvedNode::Condition { .. } => self.condition_node(station),
            ResolvedNode::Array(arguments) => self.optimize(arguments),
            ResolvedNode::Return(value) => self.optimize_node(value),
            _ => {}
        }
    }

    fn builtin_node(&mut self, station: &mut ResolvedNode) {
        let ResolvedNode::BuiltinCall { index, arguments: argument_nodes } = station else { unreachable!() };
        let builtin = get_builtin(*index);
        let function = match builtin.function {
            BuiltinFunction::Math(function) | BuiltinFunction::Compare(function) | BuiltinFunction::Casting(function) | BuiltinFunction::Introspection(function) => function,
            _ => return
        };
        let mut arguments = Vec::with_capacity(argument_nodes.len());
        let mut only_have_literal_node = true;
        for argument in argument_nodes {
            arguments.push(match argument {
                ResolvedNode::Literal(LightValue::Float(value)) => Value::Float(*value),
                ResolvedNode::Literal(LightValue::Integer(value)) => Value::Integer(*value),
                ResolvedNode::HeavyLiteral(Value::String(value)) => Value::String(value.clone()),
                ResolvedNode::Condition { .. } => { self.condition_node(argument); only_have_literal_node = false; continue }
                ResolvedNode::BuiltinCall { .. } => {
                    self.builtin_node(argument);
                    match argument {
                        ResolvedNode::Literal(LightValue::Float(value)) => Value::Float(*value),
                        ResolvedNode::Literal(LightValue::Integer(value)) => Value::Integer(*value),
                        ResolvedNode::HeavyLiteral(Value::String(value)) => Value::String(value.clone()),
                        _ => { only_have_literal_node = false; continue }
                    }
                }
                _ => { only_have_literal_node = false; continue }
            })
        }
        if !only_have_literal_node { return }
        *station = match function(&arguments) {
            Ok(Value::Boolean(value)) => ResolvedNode::Literal(LightValue::Boolean(value)),
            Ok(Value::Float(value)) => ResolvedNode::Literal(LightValue::Float(value)),
            Ok(Value::Integer(value)) => ResolvedNode::Literal(LightValue::Integer(value)),
            Ok(Value::String(value)) => ResolvedNode::HeavyLiteral(Value::String(value)),
            Ok(_) => todo!(),
            Err(error) => { self.error_handler.push_error(error); return }
        }
    }

    fn condition_node(&mut self, station: &mut ResolvedNode) {
        let ResolvedNode::Condition { branches, final_branch } = station else { unreachable!() };
        for branch in branches.iter_mut() {
            self.optimize(&mut branch.condition);
            self.optimize(&mut branch.body);
        }
        let mut have_condition_always_true = false;
        branches.retain(|branch| {
            if have_condition_always_true { return false }
            match branch.condition.last().unwrap() {
                ResolvedNode::Literal(LightValue::Boolean(boolean)) => {
                    if *boolean { have_condition_always_true = true }
                    *boolean
                }
                _ => true
            }
        });
        if have_condition_always_true {
            *final_branch = vec![]
        }
    }
}

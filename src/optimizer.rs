use crate::node::{TypedNode, TypedAST};
use crate::builtins::{get_builtin, BuiltinFunction};
use crate::value::{LightValue, HeavyValue, Value};
use crate::error_handler::ErrorHandler;

pub struct Optimizer {
    error_handler: ErrorHandler,
}

impl Optimizer {
    pub fn new(error_handler: ErrorHandler) -> Self {
        Self { error_handler }
    }

    pub fn optimizer(mut self, mut ast: TypedAST) -> (TypedAST, ErrorHandler) {
        for pipeline in &mut ast.nodes {
            let TypedNode::Pipeline(stations) = pipeline else {unreachable!()};
            self.optimize(stations);
        }
        (ast, self.error_handler)
    }

    fn optimize(&mut self, stations: &mut Vec<TypedNode>) {
        for station in stations {
            match station {
                TypedNode::BuiltinCall { .. } => self.builtin_node(station),
                TypedNode::DefineFunction { body: TypedAST { nodes, .. }, .. } => {
                    for node in nodes {
                        let TypedNode::Pipeline(pipeline) = node else { unreachable!() };
                        if let Some(index) = pipeline.iter().position(|node| matches!(node, TypedNode::Return(_))) {
                            pipeline.truncate(index + 1);
                        }
                        self.optimize(pipeline)
                    }
                },
                TypedNode::Call { arguments, .. } => self.optimize(arguments),
                TypedNode::Pipeline(station) => self.optimize(station),
                TypedNode::Condition { .. } => self.condition_node(station),
                TypedNode::Array(arguments) => self.optimize(arguments),
                _ => {}
            }
        }
    }

    fn builtin_node(&mut self, station: &mut TypedNode) {
        let TypedNode::BuiltinCall { index, arguments: argument_nodes, .. } = station else { unreachable!() };
        let builtin = get_builtin(*index);
        let function = match builtin.function {
            BuiltinFunction::Math(function) | BuiltinFunction::Compare(function) | BuiltinFunction::Casting(function) | BuiltinFunction::Introspection(function) => function,
            _ => return
        };
        let mut arguments = Vec::with_capacity(argument_nodes.len());
        let mut only_have_literal_node = true;
        for argument in argument_nodes {
            arguments.push(match argument {
                TypedNode::Literal(LightValue::Float(value)) => Value::Float(*value),
                TypedNode::Literal(LightValue::Integer(value)) => Value::Integer(*value),
                TypedNode::HeavyLiteral(HeavyValue::String(value)) => Value::String(value.clone()),
                TypedNode::Condition { .. } => { self.condition_node(argument); only_have_literal_node = false; continue }
                TypedNode::BuiltinCall { .. } => {
                    self.builtin_node(argument);
                    match argument {
                        TypedNode::Literal(LightValue::Float(value)) => Value::Float(*value),
                        TypedNode::Literal(LightValue::Integer(value)) => Value::Integer(*value),
                        TypedNode::HeavyLiteral(HeavyValue::String(value)) => Value::String(value.clone()),
                        _ => { only_have_literal_node = false; continue }
                    }
                }
                _ => { only_have_literal_node = false; continue }
            })
        }
        if !only_have_literal_node { return }
        *station = match function(&arguments) {
            Ok(Value::Boolean(value)) => TypedNode::Literal(LightValue::Boolean(value)),
            Ok(Value::Float(value)) => TypedNode::Literal(LightValue::Float(value)),
            Ok(Value::Integer(value)) => TypedNode::Literal(LightValue::Integer(value)),
            Ok(Value::String(value)) => TypedNode::HeavyLiteral(HeavyValue::String(value)),
            Ok(_) => todo!(),
            Err(error) => { self.error_handler.push_error(error); return }
        }
    }

    fn condition_node(&mut self, station: &mut TypedNode) {
        let TypedNode::Condition { branches, final_branch } = station else { unreachable!() };
        for (condition, body) in branches.iter_mut() {
            self.optimize(condition);
            self.optimize(body);
        }
        let mut have_condition_always_true = false;
        branches.retain(|(condition, _)| {
            if have_condition_always_true { return false }
            match condition.last().unwrap() {
                TypedNode::Literal(LightValue::Boolean(boolean)) => {
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

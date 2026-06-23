use crate::node::{ResolvedNode, AST};
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

    pub fn optimizer(mut self, mut ast: AST) -> (AST, ErrorHandler) {
        for pipeline in &mut ast.nodes {
            let ResolvedNode::Pipeline(stations) = pipeline else {unreachable!()};
            self.optimize(stations);
        }
        (ast, self.error_handler)
    }

    fn optimize(&mut self, stations: &mut Vec<ResolvedNode>) {
        for station in stations {
            let ResolvedNode::BuiltinCall { index, arguments: argument_nodes } = station else { continue } ;
            let builtin = get_builtin(*index);
            let function = match builtin.function {
                BuiltinFunction::Math(function) | BuiltinFunction::Compare(function) | BuiltinFunction::Casting(function) | BuiltinFunction::Introspection(function) => function,
                _ => continue
            };
            let mut arguments = Vec::with_capacity(argument_nodes.len());
            let mut skip = false;
            for argument in argument_nodes {
                arguments.push(match argument {
                    ResolvedNode::Literal(LightValue::Float(value)) => Value::Float(*value),
                    ResolvedNode::Literal(LightValue::Integer(value)) => Value::Integer(*value),
                    ResolvedNode::HeavyLiteral(HeavyValue::String(value)) => Value::String(value.clone()),
                    _ => { skip = true; break }
                })
            }
            if skip { continue }
            *station = match function(&arguments) {
                Ok(Value::Boolean(value)) => ResolvedNode::Literal(LightValue::Boolean(value)),
                Ok(Value::Float(value)) => ResolvedNode::Literal(LightValue::Float(value)),
                Ok(Value::Integer(value)) => ResolvedNode::Literal(LightValue::Integer(value)),
                Ok(Value::String(value)) => ResolvedNode::HeavyLiteral(HeavyValue::String(value)),
                Ok(_) => todo!(),
                Err(error) => { self.error_handler.errors.push(error); continue }
            }
        }
    }
}

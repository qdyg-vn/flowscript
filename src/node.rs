use crate::value::{Kind, LightValue, HeavyValue};

#[derive(Debug)]
pub enum Node {
    Literal(LightValue),
    HeavyLiteral(HeavyValue),
    Symbol(String),
    Apply {
        operator: Box<Node>,
        arguments: Vec<Node>,
    },
    Pipeline(Vec<Node>),
    RelativeReference(u16, u16),
    Variable(String),
    Assignment(String),
    HardAssignment(String, Kind),
    DefineFunction {
        operator: String,
        arguments: Vec<Node>,
        body: Vec<Node>,
    },
    Condition {
        branches: Vec<(Vec<Node>, Vec<Node>)>,
        final_branch: Vec<Node>,
    },
    Return(Box<Node>),
    Array(Vec<Node>),
}
use crate::value::{Kind, LightValue, HeavyValue};

#[derive(Debug)]
pub enum Node {
    Literal(LightValue),
    HeavyLiteral(HeavyValue),
    Apply {
        operator: String,
        arguments: Vec<Node>,
    },
    Pipeline(Vec<Node>),
    RelativeReference(u16, u16),
    Variable(String),
    SoftAssignment(String),
    Assignment(String, Kind),
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

#[derive(Debug)]
pub enum ResolvedNode {
    Literal(LightValue),
    HeavyLiteral(HeavyValue),
    BuiltinCall {
        index: u16,
        arguments: Vec<ResolvedNode>,
    },
    Call {
        scope: u16,
        index: u16,
        arguments: Vec<ResolvedNode>,
    },
    Pipeline(Vec<ResolvedNode>),
    RelativeReference(u16, u16),
    Variable(u16),
    Assignment(u16, Kind),
    DefineFunction {
        index: u16,
        body: AST,
    },
    Condition {
        branches: Vec<(Vec<ResolvedNode>, Vec<ResolvedNode>)>,
        final_branch: Vec<ResolvedNode>,
    },
    Return(Box<ResolvedNode>),
    Array(Vec<ResolvedNode>),
}

#[derive(Debug)]
pub struct AST {
    pub nodes: Vec<ResolvedNode>,
    pub arity: u16,
}
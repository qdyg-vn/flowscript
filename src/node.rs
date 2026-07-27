use crate::value::{Kind, LightValue, HeavyValue, ParentKind};

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
        signature_index: u32,
    },
    Pipeline(Vec<ResolvedNode>),
    RelativeReference(u16, u16),
    Variable(u16, Kind),
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
pub enum TypedNode {
    Literal(LightValue),
    HeavyLiteral(HeavyValue),
    BuiltinCall {
        index: u16,
        arguments: Vec<TypedNode>,
        result: ParentKind,
    },
    Call {
        scope: u16,
        index: u16,
        arguments: Vec<TypedNode>,
    },
    Pipeline(Vec<TypedNode>),
    RelativeReference(u16, u16, ParentKind),
    Variable(u16, Kind),
    Assignment(u16, Kind),
    DefineFunction {
        index: u16,
        body: TypedAST,
    },
    Condition {
        branches: Vec<(Vec<TypedNode>, Vec<TypedNode>)>,
        final_branch: Vec<TypedNode>,
    },
    Return(Box<TypedNode>),
    Array(Vec<TypedNode>),
}

#[derive(Debug)]
pub struct AST {
    pub nodes: Vec<ResolvedNode>,
    pub arity: u8,
    pub variables_count: u16,
}

#[derive(Debug)]
pub struct TypedAST {
    pub nodes: Vec<TypedNode>,
    pub arity: u8,
    pub variables_count: u16,
}

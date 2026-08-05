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
        parameters: Vec<Node>,
        body: Vec<Node>,
        result: Kind,
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
    Variable(u16, u32),
    SoftAssignment(u16, u32),
    Assignment(u16, u32, Kind),
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
        result: Kind,
    },
    Call {
        scope: u16,
        index: u16,
        arguments: Vec<TypedNode>,
        result: Kind,
    },
    Pipeline(Vec<TypedNode>),
    RelativeReference(u16, u16, Kind),
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
    Add(Vec<TypedNode>, Kind, Kind),
    Minus(Vec<TypedNode>, Kind, Kind),
    Multiply(Vec<TypedNode>, Kind, Kind),
    Equal(Vec<TypedNode>, Kind, Kind),
    LessThan(Vec<TypedNode>, Kind, Kind),
    GreaterThan(Vec<TypedNode>, Kind, Kind),
    LessThanOrEqual(Vec<TypedNode>, Kind, Kind),
    GreaterThanOrEqual(Vec<TypedNode>, Kind, Kind),
    NotEqual(Vec<TypedNode>, Kind, Kind),
}

#[derive(Debug)]
pub struct AST {
    pub nodes: Vec<ResolvedNode>,
    pub arity: u8,
    pub variables_count: u16,
    pub define_function_count: u16,
}

#[derive(Debug)]
pub struct TypedAST {
    pub nodes: Vec<TypedNode>,
    pub arity: u8,
    pub variables_count: u16,
}

use crate::value::{Kind, LightValue, Value};

#[derive(Debug)]
pub struct ConditionBranch<N> {
    pub condition: Vec<N>,
    pub body: Vec<N>
}

#[derive(Debug)]
pub enum Node {
    Literal(LightValue),
    HeavyLiteral(Value),
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
        branches: Vec<ConditionBranch<Node>>,
        final_branch: Vec<Node>,
    },
    Return(Box<Node>),
    Array(Vec<Node>),
}

#[derive(Debug)]
pub enum ResolvedNode {
    Literal(LightValue),
    HeavyLiteral(Value),
    BuiltinCall {
        index: u16,
        arguments: Vec<ResolvedNode>,
    },
    Call {
        arguments: Vec<ResolvedNode>,
        function_index: u32,
    },
    Pipeline(Vec<ResolvedNode>),
    RelativeReference(u16, u16, u32),
    StationCapture(u16, u32),
    Variable(u16, u32),
    SoftAssignment(u16, u32),
    Assignment(u16, u32, Kind),
    DefineFunction {
        function_index: u32,
        body: AST,
    },
    Condition {
        branches: Vec<ConditionBranch<ResolvedNode>>,
        final_branch: Vec<ResolvedNode>,
    },
    Return(Box<ResolvedNode>),
    Array(Vec<ResolvedNode>),
}

#[derive(Debug)]
pub enum TypedNode {
    Literal(LightValue),
    HeavyLiteral(Value),
    BuiltinCall {
        index: u16,
        arguments: Vec<TypedNode>,
        result: Kind,
    },
    Call {
        function_index: u32,
        arguments: Vec<TypedNode>,
        result: Kind,
    },
    Pipeline(Vec<TypedNode>),
    RelativeReference(u16, u16, Kind),
    Variable(u16, Kind),
    Assignment(u16, Kind),
    StationCapture(u16, Kind),
    DefineFunction {
        function_index: u32,
        body: TypedAST,
    },
    Condition {
        branches: Vec<ConditionBranch<TypedNode>>,
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

#[derive(Debug, Default)]
pub struct AST {
    pub nodes: Vec<ResolvedNode>,
    pub variables_count: u16,
    pub arity: u8,
    pub max_relative_reference: u8,
}

#[derive(Debug, Default)]
pub struct TypedAST {
    pub nodes: Vec<TypedNode>,
    pub variables_count: u16,
    pub arity: u8,
    pub max_relative_reference: u8,
}

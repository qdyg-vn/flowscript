use crate::value::Value;

#[derive(Debug)]
pub enum Node {
    Literal(Value),
    Symbol(String),
    Apply {
        operator: Box<Node>,
        arguments: Vec<Node>,
    },
    Pipeline(Vec<Node>),
    RelativeReference(u16, u16),
    Variable(String),
    Assignment(String),
    DefineFunction {
        operator: String,
        arguments: Vec<Node>,
        body: Vec<Node>,
    },
    Condition {
        condition: Vec<Node>,
        if_body: Vec<Node>,
        else_body: Vec<Node>,
    },
    Return(Box<Node>),
}
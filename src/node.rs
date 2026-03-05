#[derive(Debug)]
pub enum Node {
    BLOCK(Vec<Node>),
    CALL {
        name: String,
        args: Vec<Node>,
    },
    FUNCTION {
        name: String,
        params: Vec<String>,
        body: Box<Node>,
    },
    IDENTIFIER(String),
    LOAD(String),
}
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Arrow,
    Boolean(bool),
    Div,
    Float(f64),
    Function,
    Identifier(String),
    Int(i64),
    LeftParen,
    Minus,
    Mul,
    Plus,
    RightParen,
    Semicolon,
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct  Token {
    pub start: usize,
    pub end: usize,
    pub kind: TokenType
}

impl Token {
    pub fn new(start: usize, end: usize, kind: TokenType) -> Token {
        Self {start, end, kind}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Arrow,
    Boolean(bool),
    Div,
    Float(f64),
    Function,
    Identifier(String),
    Macro(String),
    Int(i64),
    LeftParen,
    Minus,
    Mul,
    Plus,
    RightParen,
    Semicolon,
    String(String),
}

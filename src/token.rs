#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub kind: TokenType,
}

impl Token {
    pub fn new(start: usize, end: usize, kind: TokenType) -> Token {
        Self { start, end, kind }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Arrow,
    Boolean(bool),
    Div,
    Float(f64),
    Function,
    Nil,
    Identifier(String),
    Macro(String),
    Integer(i64),
    LeftParen,
    Minus,
    Mul,
    Plus,
    RelativeReference(u16, u16),
    RightParen,
    Semicolon,
    String(String),
    Variable(String),
    DefineFunction,
    LeftBrace,
    RightBrace,
    If,
    Else,
    Return,
    LeftBracket,
    RightBracket,
    Colon,
    Kind(String),
}

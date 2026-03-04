pub struct Token {
    pub ty: TokenType,
    pub value: String,
}

impl Token {
    pub fn new(ty: TokenType, value: String) -> Token {
        Self {ty, value}
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    INT = 0,
    FLOAT = 1,
    STRING = 2,
    BOOLEAN = 3,
    LPAREN = 4,
    RPAREN = 5,
    PLUS = 6,
    MINUS = 7,
    MUL = 8,
    DIV = 9,
    SEMICOLON = 10,
    ARROW = 11,
    FUNCTION = 12,
    IDENTIFIER = 13,
    EOF = 14
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    ARROW,
    BOOLEAN(bool),
    DIV,
    EOF,
    FLOAT(f64),
    FUNCTION,
    IDENTIFIER(String),
    INT(i64),
    LPAREN,
    MINUS,
    MUL,
    PLUS,
    RPAREN,
    SEMICOLON,
    STRING(String),
}

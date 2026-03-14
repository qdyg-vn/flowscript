#[derive(Debug)]
pub enum Token<'source_code> {
    Arrow,
    Boolean(bool),
    Div,
    Float(f64),
    Function,
    Identifier(&'source_code str),
    Macro(&'source_code str),
    Int(i64),
    LeftParen,
    Minus,
    Mul,
    Plus,
    RightParen,
    Semicolon,
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Values {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
}
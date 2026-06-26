use std::{fmt, process::exit};
use crate::token::Token;
use crate::value::{Kind, Value};

#[derive(Default)]
pub struct ErrorHandler {
    pub errors: Vec<Error>,
}

impl ErrorHandler {
    pub fn fatal(&mut self, error: Error) {
        println!("{}", error);
        exit(1)
    }

    pub fn report_exit(&mut self) {
        for error in &self.errors {
            println!("{}", error)
        }
        exit(1)
    }
}

pub enum Error {
    LexicalError(LexicalError),
    SyntaxError(SyntaxError),
    SemanticError(SemanticError),
    RuntimeError(RuntimeError),
    TypeError(TypeError),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::LexicalError(error) => write!(formatter, "{}", error),
            Error::SyntaxError(error) => write!(formatter, "{}", error),
            Error::SemanticError(error) => write!(formatter, "{}", error),
            Error::RuntimeError(error) => write!(formatter, "{}", error),
            Error::TypeError(error) => write!(formatter, "{}", error),
        }
    }
}


pub struct LexicalError {
    pub line: usize,
    pub column: usize,
    pub kind: LexicalErrorType,
}


impl fmt::Display for LexicalError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let code = match &self.kind {
            LexicalErrorType::InvalidCharacter(_) => 1,
            LexicalErrorType::DecimalPoints(_) => 2,
            LexicalErrorType::MissingClosingQuote(_) => 3,
            LexicalErrorType::MultipleUnderscores(_) => 4,
            LexicalErrorType::FloatRelativeReferences(_) => 5,
        };
        writeln!(formatter, "\x1b[31;1m[Error FSCC1{:0>3}]\x1b[0m {}", code, self.kind)?;
        write!(formatter, "\x1b[38;2;143;255;46m  --> Line: {} Column: {}\n\x1b[0m", self.line, self.column)
    }
}

pub enum LexicalErrorType {
    InvalidCharacter(String),
    DecimalPoints(String),
    MissingClosingQuote(String),
    MultipleUnderscores(String),
    FloatRelativeReferences(String)
}

impl fmt::Display for LexicalErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexicalErrorType::InvalidCharacter(character) => {
                write!(formatter, "Invalid character: {}", character)
            }
            LexicalErrorType::DecimalPoints(number) => {
                write!(formatter, "Multiple decimal points: {}", number)
            }
            LexicalErrorType::MissingClosingQuote(string) => {
                write!(formatter, "Missing closing quote: {}", string)
            }
            LexicalErrorType::MultipleUnderscores(string) => {
                write!(formatter, "A relative reference can only have one underscore: {}", string)
            }
            LexicalErrorType::FloatRelativeReferences(string) => {
                write!(formatter, "Relative references cannot have x or y as floats: {}", string)
            }
        }
    }
}

pub struct SyntaxError {
    pub line: usize,
    pub column: usize,
    pub kind: SyntaxErrorType,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let code = match &self.kind {
            SyntaxErrorType::UnimplementedToken(_) => 1,
            SyntaxErrorType::MissingFunctionName => 2,
            SyntaxErrorType::RedundantFunctionDefinition => 3,
            SyntaxErrorType::MissingLeftParen => 4,
            SyntaxErrorType::MissingLeftBrace => 5,
            SyntaxErrorType::NoStationBeforePipeline => 6,
            SyntaxErrorType::NoStationAfterPipeline => 7,
            SyntaxErrorType::MissingTypeIdentity => 8,
        };
        writeln!(formatter, "\x1b[31;1m[Error FSCC3{:0>3}]\x1b[0m {}", code, self.kind)?;
        write!(formatter, "\x1b[38;2;143;255;46m  --> Line: {} Column: {}\n\x1b[0m", self.line, self.column)
    }
}

pub enum SyntaxErrorType {
    UnimplementedToken(Token),
    MissingFunctionName,
    RedundantFunctionDefinition,
    MissingLeftParen,
    MissingLeftBrace,
    NoStationBeforePipeline,
    NoStationAfterPipeline,
    MissingTypeIdentity,
}

impl fmt::Display for SyntaxErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyntaxErrorType::UnimplementedToken(token) => write!(formatter, "Unimplemented token: {:?}", token),
            SyntaxErrorType::MissingFunctionName => write!(formatter, "Function need a name!"),
            SyntaxErrorType::RedundantFunctionDefinition => write!(formatter, "There is one redundant function definition"),
            SyntaxErrorType::MissingLeftParen => write!(formatter, "Behind operator need a left paren!"),
            SyntaxErrorType::MissingLeftBrace => write!(formatter, "Behind operator need a left brace!"),
            SyntaxErrorType::NoStationBeforePipeline => write!(formatter, "There is no station before pipeline!"),
            SyntaxErrorType::NoStationAfterPipeline => write!(formatter, "There is no station after pipeline!"),
            SyntaxErrorType::MissingTypeIdentity => write!(formatter, "Missing type in declaration"),
        }
    }
}

pub struct SemanticError {
    pub kind: SemanticErrorType
}

impl fmt::Display for SemanticError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let code = match &self.kind {
            SemanticErrorType::UndefinedIdentifier(_) => 1,
            SemanticErrorType::DuplicateParameter(_) => 2,
        };
        writeln!(formatter, "\x1b[31;1m[Error FSCC5{:0>3}]\x1b[0m {}", code, self.kind)
    }
}

pub enum SemanticErrorType {
    UndefinedIdentifier(String),
    DuplicateParameter(String),
}

impl fmt::Display for SemanticErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SemanticErrorType::UndefinedIdentifier(name) => write!(formatter, "Cannot find {} in symbol table", name),
            SemanticErrorType::DuplicateParameter(name) => write!(formatter, "Identifier '{}' is bound more than once", name)
        }
    }
}

pub struct RuntimeError {
    pub kind: RuntimeErrorType,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let code = match &self.kind {
            RuntimeErrorType::MissingStation(_) => 1,
            RuntimeErrorType::OutOfBounds(_, _) => 2,
            RuntimeErrorType::InsufficientOperands(_) => 3,
            RuntimeErrorType::ParseError(_, _) => 4,
        };
        writeln!(formatter, "\x1b[31;1m[Error FSCC7{:0>3}]\x1b[0m {}", code, self.kind)
    }
}

pub enum RuntimeErrorType {
    MissingStation(u16),
    OutOfBounds(usize, usize),
    InsufficientOperands(String),
    ParseError(String, String)
}

impl fmt::Display for RuntimeErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeErrorType::MissingStation(x) => write!(formatter, "There is no station at position {}", x),
            RuntimeErrorType::OutOfBounds(start, end) => write!(formatter, "Stack out of bounds: start {} end {}", start, end),
            RuntimeErrorType::InsufficientOperands(function) => write!(formatter, "{} requires at least one operand", function),
            RuntimeErrorType::ParseError(value, kind) => write!(formatter, "Cannot convert {} to {}", value, kind),
        }
    }
}

pub struct TypeError {
    pub kind: TypeErrorType
}

impl fmt::Display for TypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let code = match &self.kind {
            TypeErrorType::NotSequence(_) => 1,
            TypeErrorType::TypeMismatch(_, _) => 2,
            TypeErrorType::InvalidOperand(_) => 3,
            TypeErrorType::InvalidUnaryOperand(_) => 4,
            TypeErrorType::AssignTypeMismatch(_, _) => 5,
            TypeErrorType::ArityMismatch(_, _) => 6,
            TypeErrorType::NotAFunction(_) => 7,
        };
        writeln!(formatter, "\x1b[31;1m[Error FSCC8{:0>3}]\x1b[0m {}", code, self.kind)
    }
}

pub enum TypeErrorType {
    NotSequence(Value),
    TypeMismatch(Kind, Kind),
    InvalidOperand(String),
    InvalidUnaryOperand(String),
    AssignTypeMismatch(Kind, Kind),
    ArityMismatch(usize, usize),
    NotAFunction(String),
}

impl fmt::Display for TypeErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeErrorType::NotSequence(station) => write!(formatter, "{} is not a sequence", station),
            TypeErrorType::TypeMismatch(accumulator, x) => write!(formatter, "Expected {} found {}", accumulator, x),
            TypeErrorType::InvalidOperand(message) | TypeErrorType::InvalidUnaryOperand(message) => write!(formatter, "{}", message),
            TypeErrorType::AssignTypeMismatch(received_kind, required_kind) => write!(formatter, "Type '{}' is not assignable to type '{}'", received_kind, required_kind),
            TypeErrorType::ArityMismatch(received_arity, required_arity) => write!(formatter, "Function expects {} arguments, but got {}", required_arity, received_arity),
            TypeErrorType::NotAFunction(name) => write!(formatter, "{} is not a function", name)
        }
    }
}

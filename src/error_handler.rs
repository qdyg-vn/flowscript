use std::{fmt, process::exit};
use crate::token::Token;

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
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::LexicalError(error) => write!(formatter, "{}", error),
            Error::SyntaxError(error) => write!(formatter, "{}", error),
            Error::SemanticError(error) => write!(formatter, "{}", error),
            Error::RuntimeError(error) => write!(formatter, "{}", error),
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
        write!(formatter, "\x1b[31;1m[Error FSCC1{:0>3}]\x1b[0m {}\n", code, self.kind)?;
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
        };
        write!(formatter, "\x1b[31;1m[Error FSCC3{:0>3}]\x1b[0m {}\n", code, self.kind)?;
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
        };
        write!(formatter, "\x1b[31;1m[Error FSCC5{:0>3}]\x1b[0m {}\n", code, self.kind)
    }
}

pub enum SemanticErrorType {
    UndefinedIdentifier(String),
}

impl fmt::Display for SemanticErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SemanticErrorType::UndefinedIdentifier(name) => write!(formatter, "Cannot find {} in symbol table", name),
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
        };
        write!(formatter, "\x1b[31;1m[Error FSCC7{:0>3}]\x1b[0m {}\n", code, self.kind)
    }
}

pub enum RuntimeErrorType {
    MissingStation(u16),
    OutOfBounds(usize, usize),
}

impl fmt::Display for RuntimeErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeErrorType::MissingStation(x) => write!(formatter, "There is no station at position {}", x),
            RuntimeErrorType::OutOfBounds(start, end) => write!(formatter, "Stack out of bounds: start {} end {}", start, end),
        }
    }
}

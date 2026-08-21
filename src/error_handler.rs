use std::{fmt, process::exit};
use crate::token::Token;
use crate::value::{Kind, Value};

#[derive(Default)]
pub struct ErrorHandler {
    pub errors: Vec<Error>,
}

impl ErrorHandler {
    pub fn fatal(&mut self, error: Error) -> ! {
        println!("{}", error);
        exit(1)
    }

    pub fn report_exit(&mut self) -> ! {
        for error in &self.errors {
            println!("{}", error)
        }
        exit(1)
    }

    pub fn push_error<E>(&mut self, error: E) where E: Into<Error> {
        self.errors.push(error.into())
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
            Self::LexicalError(error) => write!(formatter, "{}", error),
            Self::SyntaxError(error) => write!(formatter, "{}", error),
            Self::SemanticError(error) => write!(formatter, "{}", error),
            Self::RuntimeError(error) => write!(formatter, "{}", error),
            Self::TypeError(error) => write!(formatter, "{}", error),
        }
    }
}

impl From<LexicalError> for Error {
    fn from(error: LexicalError) -> Self {
        Self::LexicalError(error)
    }
}

impl From<SyntaxError> for Error {
    fn from(error: SyntaxError) -> Self {
        Self::SyntaxError(error)
    }
}

impl From<SemanticError> for Error {
    fn from(error: SemanticError) -> Self {
        Self::SemanticError(error)
    }
}

impl From<RuntimeError> for Error {
    fn from(error: RuntimeError) -> Self {
        Self::RuntimeError(error)
    }
}

impl From<TypeError> for Error {
    fn from(error: TypeError) -> Self {
        Self::TypeError(error)
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
            Self::InvalidCharacter(character) => {
                write!(formatter, "Invalid character: {}", character)
            }
            Self::DecimalPoints(number) => {
                write!(formatter, "Multiple decimal points: {}", number)
            }
            Self::MissingClosingQuote(string) => {
                write!(formatter, "Missing closing quote: {}", string)
            }
            Self::MultipleUnderscores(string) => {
                write!(formatter, "A relative reference can only have one underscore: {}", string)
            }
            Self::FloatRelativeReferences(string) => {
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
            SyntaxErrorType::MissingCondition => 9,
            SyntaxErrorType::MissingRightParen => 10,
            SyntaxErrorType::MissingEndKeyword => 11,
            SyntaxErrorType::MissingRightBracket => 12,
            SyntaxErrorType::RedundantFunction => 13,
            SyntaxErrorType::InvalidTypeError => 14,
            SyntaxErrorType::MissingFunctionBody => 15,
            SyntaxErrorType::RedundantCondition => 16,
            SyntaxErrorType::MissingConditionBody => 17,
            SyntaxErrorType::MissingType(_) => 18,
            SyntaxErrorType::MissingDoKeyword => 19,
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
    MissingCondition,
    MissingRightParen,
    MissingEndKeyword,
    MissingRightBracket,
    RedundantFunction,
    InvalidTypeError,
    MissingFunctionBody,
    RedundantCondition,
    MissingConditionBody,
    MissingType(String),
    MissingDoKeyword,
}

impl fmt::Display for SyntaxErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnimplementedToken(token) => write!(formatter, "Unimplemented token: {:?}", token),
            Self::MissingFunctionName => write!(formatter, "Function needs a name!"),
            Self::RedundantFunctionDefinition => write!(formatter, "There is one redundant function definition"),
            Self::MissingLeftParen => write!(formatter, "Behind operator needs a left paren!"),
            Self::MissingLeftBrace => write!(formatter, "Behind condition needs a left brace!"),
            Self::NoStationBeforePipeline => write!(formatter, "There is no station before pipeline!"),
            Self::NoStationAfterPipeline => write!(formatter, "There is no station after pipeline!"),
            Self::MissingTypeIdentity => write!(formatter, "Missing type in declaration"),
            Self::MissingCondition => write!(formatter, "The conditional expression is empty!"),
            Self::MissingRightParen => write!(formatter, "Behind function arguments needs a right paren!"),
            Self::MissingEndKeyword => write!(formatter, "Missing 'end' after function body!"),
            Self::MissingRightBracket => write!(formatter, "Behind array elements needs a right bracket!"),
            Self::RedundantFunction => write!(formatter, "There is one redundant function"),
            Self::InvalidTypeError => write!(formatter, "Behind ':' needs a valid type!"),
            Self::MissingFunctionBody => write!(formatter, "Function needs a body!"),
            Self::RedundantCondition => write!(formatter, "There is one redundant condition"),
            Self::MissingConditionBody => write!(formatter, "Condition needs a body!"),
            Self::MissingDoKeyword => write!(formatter, "Missing 'do' after function arguments!"),
            Self::MissingType(name) => write!(formatter, "Missing type for parameter '{}'", name),
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
            SemanticErrorType::NegativeXCoordinate(_) => 3,
            SemanticErrorType::MissingStation(_) => 4,
            SemanticErrorType::RelativeReferenceNotInPipeline => 5,
        };
        writeln!(formatter, "\x1b[31;1m[Error FSCC5{:0>3}]\x1b[0m {}", code, self.kind)
    }
}

pub enum SemanticErrorType {
    UndefinedIdentifier(String),
    DuplicateParameter(String),
    NegativeXCoordinate(u16),
    MissingStation(u16),
    RelativeReferenceNotInPipeline,
}

impl fmt::Display for SemanticErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UndefinedIdentifier(name) => write!(formatter, "Cannot find {} in symbol table", name),
            Self::DuplicateParameter(name) => write!(formatter, "Identifier '{}' is bound more than once", name),
            Self::NegativeXCoordinate(x) => write!(formatter, "x must be positive when y is zero: {}", x),
            Self::MissingStation(x) => write!(formatter, "There is no station at position {}", x),
            Self::RelativeReferenceNotInPipeline => write!(formatter, "Cannot use relative reference outside a pipeline"),
        }
    }
}

pub struct RuntimeError {
    pub kind: RuntimeErrorType,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let code = match &self.kind {
            RuntimeErrorType::OutOfBounds(_, _) => 1,
            RuntimeErrorType::InsufficientOperands(_) => 2,
            RuntimeErrorType::ParseError(_, _) => 3,
        };
        writeln!(formatter, "\x1b[31;1m[Error FSCC7{:0>3}]\x1b[0m {}", code, self.kind)
    }
}

pub enum RuntimeErrorType {
    OutOfBounds(usize, usize),
    InsufficientOperands(String),
    ParseError(String, String)
}

impl fmt::Display for RuntimeErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OutOfBounds(start, end) => write!(formatter, "Stack out of bounds: start {} end {}", start, end),
            Self::InsufficientOperands(function) => write!(formatter, "{} requires at least one operand", function),
            Self::ParseError(value, kind) => write!(formatter, "Cannot convert {} to {}", value, kind),
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
            Self::NotSequence(station) => write!(formatter, "{} is not a sequence", station),
            Self::TypeMismatch(accumulator, x) => write!(formatter, "Expected {} found {}", accumulator, x),
            Self::InvalidOperand(message) | TypeErrorType::InvalidUnaryOperand(message) => write!(formatter, "{}", message),
            Self::AssignTypeMismatch(received_kind, required_kind) => write!(formatter, "Type '{}' is not assignable to type '{}'", received_kind, required_kind),
            Self::ArityMismatch(received_arity, required_arity) => write!(formatter, "Function expects {} arguments, but got {}", required_arity, received_arity),
            Self::NotAFunction(name) => write!(formatter, "{} is not a function", name),
        }
    }
}

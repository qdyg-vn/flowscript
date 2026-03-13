use std::fmt;

#[derive(Default)]
pub struct ErrorHandler {
    pub errors: Vec<Error>,
}

impl ErrorHandler {
    pub fn report(&self) {
        for error in &self.errors {
            println!("{}", error);
        }
    }

}

pub struct Error {
    pub line: usize,
    pub kind: ErrorType,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let code = match &self.kind {
            ErrorType::InvalidCharacter(_) => 1,
            ErrorType::DecimalPoints(_) => 2,
            ErrorType::MissingClosingQuote(_) => 3,
        };
        write!(formatter, "\x1b[31;1m[Error FSCC{:0>4}]\x1b[0m {}\n", code, self.kind)?;
        write!(formatter, "\x1b[38;2;143;255;46m  --> Line: {}\n\x1b[0m", self.line)
    }
}

pub enum ErrorType {
    InvalidCharacter(char),
    DecimalPoints(String),
    MissingClosingQuote(String),
}

impl fmt::Display for ErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorType::InvalidCharacter(character) => {
                write!(formatter, "Invalid character: {}", character)
            }
            ErrorType::DecimalPoints(number) => {
                write!(formatter, "Multiple decimal points: {}", number)
            }
            ErrorType::MissingClosingQuote(string) => {
                write!(formatter, "Missing closing quote: {}", string)
            }
        }
    }
}

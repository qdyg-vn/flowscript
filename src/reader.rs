use crate::error_handler::{Error, ErrorType};
use std::fs::read_to_string;
use std::io::{self, Write};

pub struct FileReader {
    path: String,
}

impl FileReader {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Iterator for FileReader {
    type Item = Result<String, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match read_to_string(&self.path) {
            Ok(source_code) => Some(Ok(source_code)),
            Err(_) => todo!("Error Handle at here because we can't read file"),
        }
    }
}

pub struct Repl;

impl Iterator for Repl {
    type Item = Result<String, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        print!(">>> ");
        io::stdout().flush().ok()?;
        let mut code = String::new();
        match io::stdin().read_line(&mut code) {
            Ok(0) => None,
            Ok(_) => Some(Ok(code.trim().to_string())),
            Err(_) => todo!("Error Handle at here!")
        }
    }
}

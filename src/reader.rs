use crate::error_handler::{Error, ErrorType};
use std::fs::read;
use std::io::{self, Write, Read};

pub struct FileReader {
    path: String,
}

impl FileReader {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Iterator for FileReader {
    type Item = Result<Vec<u8>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match read(&self.path) {
            Ok(source_code) => Some(Ok(source_code)),
            Err(_) => todo!("Error Handle at here because we can't read file"),
        }
    }
}

pub struct Repl;

impl Iterator for Repl {
    type Item = Result<Vec<u8>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        print!(">>> ");
        io::stdout().flush().ok()?;
        let mut code = Vec::new();
        match io::stdin().read_to_end(&mut code) {
            Ok(0) => None,
            Ok(_) => Some(Ok(code)),
            Err(_) => todo!("Error Handle at here!")
        }
    }
}

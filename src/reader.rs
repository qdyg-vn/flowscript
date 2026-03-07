use std::fs::File;
use std::io::{self, Write};
use std::io::{BufRead, BufReader, Lines};

pub struct FileReader {
    iter: Lines<BufReader<File>>,
}

impl FileReader {
    pub fn new(path: &str) -> Self {
        let file = File::open(path).expect("Could not open file");
        let reader = BufReader::new(file);
        Self {
            iter: reader.lines(),
        }
    }
}

impl Iterator for FileReader {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()?.ok()
    }
}

pub struct Repl;

impl Iterator for Repl {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        print!(">>> ");
        io::stdout().flush().ok()?;
        let mut code = String::new();
        match io::stdin().read_line(&mut code) {
            Ok(0) => None,
            Ok(_) => Some(code.trim().to_string()),
            Err(_) => None,
        }
    }
}

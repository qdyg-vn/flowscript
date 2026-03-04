use std::fs::File;
use std::io::{BufRead, BufReader, Lines};


pub struct Reader {
    iter: Lines<BufReader<File>>
}

impl Reader {
    pub fn new(path: &str) -> Self {
        let file = File::open(path).expect("Could not open file");
        let reader = BufReader::new(file);
        Self {iter: reader.lines()}
    }
}

pub trait LineReader {
    fn read_line(&mut self) -> Option<String>;
}

impl LineReader for Reader {
    fn read_line(&mut self) -> Option<String> {
        self.iter.next()?.ok()
    }
}
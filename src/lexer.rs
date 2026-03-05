use std::process::exit;
use crate::token::Token;
use crate::reader::LineReader;

pub struct Lexer<R: LineReader> {
    reader: R,
    code: String,
    pos: usize,
}

impl<R: LineReader> Lexer<R> {
    pub fn new(reader: R) -> Self {
        Self {reader, code: String::new(), pos: 0}
    }

    fn ensure_buffer(&mut self) -> bool {
        while self.pos >= self.code.len() {
            match self.reader.read_line() {
                Some(line) => {
                    self.code = line;
                    self.pos = 0;
                    if !self.code.is_empty() { return true }
                }
                None => return false
            }
        }
        true
    }

    fn advance(&mut self, steps: usize) -> Option<char> {
        let character = self.peek();
        if character.is_some() { self.pos += steps }
        character
    }

    fn peek(&mut self) -> Option<char> {
        if self.ensure_buffer() {
            self.code.as_bytes().get(self.pos).map(|c| *c as char)
        } else { None }
    }

    fn string_collector(&mut self, quotation_mark: char) -> Token {
        let start = self.pos;
        loop {
            match self.advance(1) {
                Some(c) if c == quotation_mark => break,
                Some(_) => continue,
                None => { eprintln!("Missing closing quote!"); exit(3) }
            }
        }
        Token::STRING(self.code[start..self.pos - 1].to_string())
    }

    fn number_collector(&mut self, _first_number: char) -> Token {
        let start = self.pos - 1; // Because pos now in second number so we need -1
        let mut has_dot = false;
        while let Some(character) = self.peek() {
            match character {
                '0'..='9' => (),
                '.' => {
                    if has_dot { eprintln!("Multiple Decimal Points!"); exit(2); }
                    has_dot = true;
                }
                _ => break,
            }
            self.advance(1);
        }
        let value = &self.code[start..self.pos];
        if has_dot { Token::FLOAT(value.parse().unwrap()) }
        else { Token::INT(value.parse().unwrap()) }
    }
}

impl<R: LineReader> Iterator for Lexer<R> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(character) = self.advance(1) {
            match character {
                '"' | '\'' => return Some(self.string_collector(character)),
                '0'..='9' => return Some(self.number_collector(character)),
                ' ' | '\t' | '\n' => continue,
                _ => {
                    eprintln!("Invalid Character!");
                    exit(1);
                }
            }
        }
        None
    }
}
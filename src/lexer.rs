use crate::error_handler::{Error, ErrorType};
use crate::token::Token;

pub struct Lexer<R: Iterator<Item = String>> {
    reader: R,
    code: String,
    pos: usize,
    line: usize,
}

impl<R: Iterator<Item = String>> Lexer<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            code: String::new(),
            pos: 0,
            line: 0,
        }
    }

    fn ensure_buffer(&mut self) -> bool {
        while self.pos >= self.code.len() {
            match self.reader.next() {
                Some(line) => {
                    self.code = line;
                    self.pos = 0;
                    self.line += 1;
                    if !self.code.is_empty() {
                        return true;
                    }
                }
                None => return false,
            }
        }
        true
    }

    fn advance(&mut self, steps: usize) -> Option<char> {
        let character = self.peek();
        if character.is_some() {
            self.pos += steps
        }
        character
    }

    fn peek(&mut self) -> Option<char> {
        if self.ensure_buffer() {
            self.code.as_bytes().get(self.pos).map(|c| *c as char)
        } else {
            None
        }
    }

    fn string_collector(&mut self, quotation_mark: char) -> Result<Token, Error> {
        let start = self.pos;
        loop {
            match self.advance(1) {
                Some(character) if character == quotation_mark => break,
                Some(_) => continue,
                None => {
                    let code = self.code[start..self.pos].to_owned();
                    return Err(self.error_collector(
                        ErrorType::MissingClosingQuote(code),
                    ));
                }
            }
        }
        Ok(Token::String(self.code[start..self.pos - 1].to_owned()))
    }

    fn number_collector(&mut self, _first_number: char) -> Result<Token, Error> {
        let start = self.pos - 1; // Because pos now in second number so we need -1
        let mut has_dot = 0;
        while let Some(character) = self.peek() {
            match character {
                '0'..='9' => (),
                '.' => {
                    has_dot += 1;
                }
                _ => break,
            }
            self.advance(1);
        }
        let value = &self.code[start..self.pos];
        match has_dot {
            0 => Ok(Token::Int(value.parse().unwrap())),
            1 => Ok(Token::Float(value.parse().unwrap())),
            _ => Err(self.error_collector(
                ErrorType::DecimalPoints(value.to_owned()),
            )),
        }
    }

    fn error_collector(&self, kind: ErrorType) -> Error {
        Error {
            line: self.line,
            kind,
        }
    }
}

impl<R: Iterator<Item = String>> Iterator for Lexer<R> {
    type Item = Result<Token, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(character) = self.advance(1) {
            match character {
                '"' | '\'' => return Some(self.string_collector(character)),
                '0'..='9' => return Some(self.number_collector(character)),
                ' ' | '\t' | '\n' => continue,
                _ => {
                    return Some(Err(self.error_collector(
                        ErrorType::InvalidCharacter(character),
                    )));
                }
            }
        }
        None
    }
}

use crate::error_handler::{Error, ErrorType};
use crate::token::Token;
use std::{iter::Peekable, str::CharIndices};

pub struct Lexer<'source_code> {
    source_code: &'source_code str,
    iter: Peekable<CharIndices<'source_code>>,
    line: usize,
}

impl<'source_code> Lexer<'source_code> {
    pub fn new(source_code: &'source_code str) -> Self {
        Self {
            source_code,
            iter: source_code.char_indices().peekable(),
            line: 0,
        }
    }

    fn string_collector(&mut self, start: usize, quotation_mark: char) -> Result<Token<'source_code>, Error> {
        loop {
            match self.iter.next() {
                Some((_, character)) if character == quotation_mark => break,
                Some(_) => continue,
                None => {
                    let code = self.source_code[start..self.source_code.len()].to_owned();
                    return Err(self.error_collector(
                        ErrorType::MissingClosingQuote(code),
                    ));
                }
            }
        }
        let end = self.iter.peek().map(|(index, _)| *index).unwrap_or(self.source_code.len());
        Ok(Token::String(&self.source_code[start..end]))
    }

    fn number_collector(&mut self, start: usize) -> Result<Token<'source_code>, Error> {
        let mut has_dot = 0;
        while let Some(character) = self.iter.peek().map(|(_, character)| character) {
            match character {
                '0'..='9' => (),
                '.' => {
                    has_dot += 1;
                }
                _ => break,
            }
            self.iter.next();
        }
        let end = self
            .iter
            .peek()
            .map(|(index, _)| *index)
            .unwrap_or(self.source_code.len());
        let value = &self.source_code[start..end];
        match has_dot {
            0 => Ok(Token::Int(value.parse().unwrap())),
            1 => Ok(Token::Float(value.parse().unwrap())),
            _ => Err(self.error_collector(ErrorType::DecimalPoints(value.to_owned()))),
        }
    }

    fn error_collector(&self, kind: ErrorType) -> Error {
        Error {
            line: self.line,
            kind,
        }
    }
}

impl<'source_code> Iterator for Lexer<'source_code> {
    type Item = Result<Token<'source_code>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((index, character)) = self.iter.next() {
            match character {
                '"' | '\'' => return Some(self.string_collector(index, character)),
                '0'..='9' => return Some(self.number_collector(index)),
                ' ' | '\t' | '\n' => continue,
                _ => {
                    return Some(Err(
                        self.error_collector(ErrorType::InvalidCharacter(character))
                    ));
                }
            }
        }
        None
    }
}

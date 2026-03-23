use crate::error_handler::{Error, ErrorType};
use crate::token::{Token, TokenType};
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

    fn string_collector(&mut self, start: usize, quotation_mark: char) -> Result<Token, Error> {
        let mut end = self.source_code.len();
        loop {
            match self.iter.next() {
                Some((index, character)) if character == quotation_mark => {
                    end = index;
                    break
                },
                Some(_) => continue,
                None => {
                    let code = self.source_code[start..end].to_owned();
                    return Err(self.error_collector(
                        ErrorType::MissingClosingQuote(code),
                    ));
                }
            }
        }
        Ok(Token::new(start + 1, end, TokenType::String(self.source_code[start..end].to_owned())
        ))
    }

    fn number_collector(&mut self, start: usize) -> Result<Token, Error> {
        let mut has_dot = 0;
        let mut has_underscore = 0;
        while let Some(character) = self.iter.peek().map(|(_, character)| character) {
            match character {
                '0'..='9' => (),
                '.' => has_dot += 1,
                '_' => has_underscore += 1,
                _ => break,
            }
            self.iter.next();
        }
        let end = self.iter.peek().map(|(index, _)| *index).unwrap_or(self.source_code.len());
        let value = &self.source_code[start..end];
        if has_underscore > 0 && has_dot > 0 {
            todo!("Relative references cannot have x or y as floats")
        }
        if has_underscore != 0 {
            if has_underscore > 1 {
                todo!("A relative reference can only have one _")}
            let mut parts = value.splitn(2, '_');
            let x = parts.next().unwrap().parse().unwrap();
            let y = parts.next().filter(|y| !y.is_empty()).unwrap_or("0").parse().unwrap();
            return Ok(Token::new(start, end, TokenType::RelativeReference(x, y)))
        }
        match has_dot {
            0 => Ok(Token::new(start, end, TokenType::Int(value.parse().unwrap()))),
            1 => Ok(Token::new(start, end, TokenType::Float(value.parse().unwrap()))),
            _ => Err(self.error_collector(ErrorType::DecimalPoints(value.to_owned()))),
        }
    }

    fn identifier_collector(&mut self, start: usize, character: char) -> Result<Token, Error> {
        if character == '-' && matches!(self.iter.peek(), Some((_, '>'))) {
            self.iter.next();
            return Ok(Token::new(start, start + 2, TokenType::Arrow))
        }
        let mut is_relative_reference = character == '_';
        loop {
            match self.iter.peek() {
                Some((_, 'a'..='z' | 'A'..='Z' | '_' | '+' | '-' | '*' | '/' | '>' | '<' | '=' | '?' | '!')) => {
                    is_relative_reference = false;
                    self.iter.next();
                },
                Some((_, '0'..='9')) => { self.iter.next(); },
                _ => break
            }
        }
        let end = self.iter.peek().map(|(index, _)| *index).unwrap_or(self.source_code.len());
        if is_relative_reference {
            let y = (&self.source_code[start + 1..end]).parse().unwrap_or(0);
            return Ok(Token::new(start, end, TokenType::RelativeReference(1, y)))
        }
        let value = self.source_code[start..end].to_owned();
        if value.ends_with('!') {
            return Ok(Token::new(start, end, TokenType::Macro(value)))
        }
        Ok(Token::new(start, end, TokenType::Identifier(value)))
    }

    fn error_collector(&self, kind: ErrorType) -> Error {
        Error {
            line: self.line,
            kind,
        }
    }
}

impl<'source_code> Iterator for Lexer<'source_code> {
    type Item = Result<Token, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((index, character)) = self.iter.next() {
            match character {
                '"' | '\'' => return Some(self.string_collector(index, character)),
                '0'..='9' => return Some(self.number_collector(index)),
                ' ' | '\t' | '\n' | ',' => continue,
                'a'..='z' | 'A'..='Z' | '_' | '+' | '-' | '*' | '/' | '>' | '<' | '=' | '?' | '!' => return Some(self.identifier_collector(index, character)),
                '(' => return Some(Ok(Token::new(index, index + 1, TokenType::LeftParen))),
                ')' => return Some(Ok(Token::new(index, index + 1, TokenType::RightParen))),
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
use crate::error_handler::{Error, LexicalError, LexicalErrorType};
use crate::token::{Token, TokenType};

pub struct Lexer<'source_code> {
    source_code: &'source_code [u8],
    position: usize,
    line: usize,
    col: usize,
}

impl<'source_code> Lexer<'source_code> {
    pub fn new(source_code: &'source_code [u8]) -> Self {
        Self {
            source_code,
            position: 0,
            line: 0,
            col: 0,
        }
    }

    fn peek(&self, steps: usize) -> Option<u8> {
        self.source_code.get(self.position + steps).cloned()
    }

    fn advance(&mut self, steps: usize) -> Option<u8> {
        if let Some(result) = self.peek(steps) {
            if result == b'\n' {
                self.line += 1;
                self.col = 1
            } else if (result & 0xC0) != 0x80 { self.col += 1 }
            self.position += steps + 1;
            return Some(result)
        }
        None
    }

    fn string_collector(&mut self, quotation_index: usize, quotation_mark: u8) -> Result<Token, Error> {
        let start = quotation_index + 1;
        let mut end = self.source_code.len();
        loop {
            match self.advance(0) {
                Some(character) if character == quotation_mark => {
                    end = self.position - 1;
                    break
                },
                Some(_) => continue,
                None => {
                    let bytes = &self.source_code[start..end];
                    let code = std::str::from_utf8(bytes).unwrap().to_owned();
                    return Err(self.error_collector(
                        LexicalErrorType::MissingClosingQuote(code),
                    ));
                }
            }
        }
        let bytes = &self.source_code[start..end];
        let string = std::str::from_utf8(bytes).unwrap();
        Ok(Token::new(start + 1, end, TokenType::String(string.to_owned())
        ))
    }

    fn number_collector(&mut self, start: usize) -> Result<Token, Error> {
        let mut has_dot = 0;
        let mut has_underscore = 0;
        while let Some(character) = self.peek(0) {
            match character {
                b'0'..=b'9' => (),
                b'.' => has_dot += 1,
                b'_' => has_underscore += 1,
                _ => break,
            }
            self.advance(0);
        }
        let end = self.position;
        let bytes = &self.source_code[start..end];
        let value = std::str::from_utf8(bytes).unwrap();
        if has_underscore > 0 && has_dot > 0 {
            return Err(self.error_collector(LexicalErrorType::FloatRelativeReferences(value.to_string())))
        }
        if has_underscore != 0 {
            if has_underscore > 1 {
                return Err(self.error_collector(LexicalErrorType::MultipleUnderscores(value.to_string())))
            }
            let mut parts = value.splitn(2, '_');
            let x = parts.next().unwrap().parse().unwrap();
            let y = parts.next().filter(|y| !y.is_empty()).unwrap_or("0").parse().unwrap();
            return Ok(Token::new(start, end, TokenType::RelativeReference(x, y)))
        }
        match has_dot {
            0 => Ok(Token::new(start, end, TokenType::Int(value.parse().unwrap()))),
            1 => Ok(Token::new(start, end, TokenType::Float(value.parse().unwrap()))),
            _ => Err(self.error_collector(LexicalErrorType::DecimalPoints(value.to_owned()))),
        }
    }

    fn identifier_collector(&mut self, start: usize, character: u8) -> Result<Token, Error> {
        if character == b'-' && matches!(self.peek(0), Some(b'>')) {
            self.advance(0);
            return Ok(Token::new(start, start + 2, TokenType::Arrow))
        }
        let mut is_relative_reference = character == b'_';
        loop {
            match self.peek(0) {
                Some(b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'+' | b'*' | b'/' | b'>' | b'<' | b'=' | b'?' | b'!') => {
                    is_relative_reference = false;
                    self.advance(0);
                },
                Some(b'-') => {
                    if self.peek(1) == Some(b'>') { break }
                    is_relative_reference = false;
                    self.advance(0);
                },
                Some(b'0'..=b'9') => { self.advance(0); },
                _ => break
            }
        }
        let end = self.position;
        let bytes = &self.source_code[start..end];
        let value = std::str::from_utf8(bytes).unwrap();
        if is_relative_reference {
            let y = value.parse().unwrap_or(0);
            return Ok(Token::new(start, end, TokenType::RelativeReference(1, y)))
        }
        match value {
            "fun" => return Ok(Token::new(start, end, TokenType::DefineFunction)),
            "if" => return Ok(Token::new(start, end, TokenType::If)),
            "else" => return Ok(Token::new(start, end, TokenType::Else)),
            "return" => return Ok(Token::new(start, end, TokenType::Return)),
            "true" => return Ok(Token::new(start, end, TokenType::Boolean(true))),
            "false" => return Ok(Token::new(start, end, TokenType::Boolean(false))),
            "nil" => return Ok(Token::new(start, end, TokenType::Nil)),
            _ => ()
        }
        if self.peek(0) != Some(b'(') {
            return Ok(Token::new(start, end, TokenType::Variable(value.to_owned())))
        }
        if value.ends_with('!') {
            return Ok(Token::new(start, end, TokenType::Macro(value.to_owned())))
        }
        Ok(Token::new(start, end, TokenType::Identifier(value.to_owned())))
    }

    fn skip_comment(&mut self) {
        if self.advance(0) == Some(b'#') {
            while let Some(token) = self.advance(0) {
                if token == b'#' && self.peek(0) == Some(b'#') {
                    self.advance(0);
                    break;
                }
            }
        } else {
            while let Some(token) = self.advance(0) && token != b'\n' {}
        }
    }

    fn error_collector(&self, kind: LexicalErrorType) -> Error {
        Error::LexicalError(LexicalError { line: self.line, kind })
    }
}

impl<'source_code> Iterator for Lexer<'source_code> {
    type Item = Result<Token, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = self.position;
            let bytes = match self.advance(0) {
                Some(bytes) => bytes,
                None => break
            };
            match bytes {
                b'"' | b'\'' => return Some(self.string_collector(start, bytes)),
                b'0'..=b'9' => return Some(self.number_collector(start)),
                b' ' | b'\t' | b'\n' | b',' => continue,
                b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'+' | b'-' | b'*' | b'/' | b'>' | b'<' | b'=' | b'?' | b'!' => return Some(self.identifier_collector(start, bytes)),
                b'(' => return Some(Ok(Token::new(start, start + 1, TokenType::LeftParen))),
                b')' => return Some(Ok(Token::new(start, start + 1, TokenType::RightParen))),
                b'{' => return Some(Ok(Token::new(start, start + 1, TokenType::LeftBrace))),
                b'}' => return Some(Ok(Token::new(start, start + 1, TokenType::RightBrace))),
                b'#' => { self.skip_comment(); continue },
                _ => {
                    return Some(Err(
                        self.error_collector(LexicalErrorType::InvalidCharacter(char::from(bytes)))
                    ));
                }
            }
        }
        None
    }
}
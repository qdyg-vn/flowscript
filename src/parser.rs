use crate::token::Token;
use crate::node::Node;

pub struct Parser<L: Iterator<Item=Token>> {
    lex: L,
    tokens: Vec<Token>,
    pos: usize
}

impl<L: Iterator<Item=Token>> Parser<L> {
    fn ensure_buffer(&mut self) -> bool {
        while self.pos >= self.tokens.len() {
            match self.lex.next() {
                Some(tokens) => {
                    self.tokens.push(tokens);
                    if !self.tokens.is_empty() { return true }
                }
                None => return false
            }
        }
        true
    }

    fn advance(&mut self, steps: usize) -> Option<Token> {
        let character = self.peek();
        if character.is_some() { self.pos += steps }
        character
    }

    fn peek(&mut self) -> Option<Token> {
        if self.ensure_buffer() {
            Some(self.tokens[self.pos].clone())
        } else { None }
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let result = Vec::new();
        while let Some(token) = self.lex.next() {
            match token {
                
            }
        }
        result
    }
}
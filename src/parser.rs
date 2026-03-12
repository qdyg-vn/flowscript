use crate::token::Token;
use crate::node::Node;
use crate::value::Value;

pub struct Parser<L: Iterator<Item=Token>> {
    lex: L,
    tokens: Vec<Token>,
    pos: usize
}

impl<L: Iterator<Item=Token>> Parser<L> {
    pub fn new(lex: L) -> Self { Self{ lex, tokens: Vec::new(), pos: 0 } }

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

    fn to_node(&mut self, token: Token) -> Node {
        match token {
            Token::Int(number) => Node::Literal(Value::Integer(number)),
            Token::Float(number) => Node::Literal(Value::Float(number)),
            Token::Identifier(identifier) => self.parse_function(identifier),
            _ => todo!("Unimplemented token: {:?}", token)
        }
    }

    fn parse_function(&mut self, token: String) -> Node {
        let operator = Box::new(Node::Symbol(token));
        if self.advance(1) != Some(Token::LeftParen) {
            todo!("Behind operator need a left paren!");
        }
        let mut arguments = Vec::new();
        while let Some(argument) = self.advance(1) {
            if argument == Token::RightParen { break }
            arguments.push(self.to_node(argument));
        }
        self.advance(1); // Skip right paren
        Node::Apply { operator, arguments }
    }

    fn parse_pipeline(&mut self, token: Token) -> Node {
        let mut stations = Vec::new();
        stations.push(self.to_node(token));
        if self.peek().is_none() || self.peek().unwrap() != Token::Arrow {
            match stations.pop() {
                Some(station) => return station,
                _ => todo!("There is no station before pipeline!")
            }
        }
        while let Some(Token::Arrow) = self.peek() {
            self.advance(1);
            if let Some(token) = self.advance(1) {
                stations.push(self.to_node(token));
            } else { todo!("We should make error_handle.rs") }
        }
        Node::Pipeline(stations)
    }
}

impl<L: Iterator<Item=Token>> Iterator for Parser<L> {
    type Item = Node;
    fn next(&mut self) -> Option<Self::Item> {
        self.advance(1).map(|token| self.parse_pipeline(token))
    }
}
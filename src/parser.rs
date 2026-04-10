use crate::error_handler::Error;
use crate::node::Node;
use crate::token::{Token, TokenType};
use crate::value::Value;
use std::rc::Rc;

pub struct Parser<L>
where
    L: Iterator<Item = Result<Token, Error>>,
{
    lex: L,
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<Error>,
}

impl<L> Parser<L>
where
    L: Iterator<Item = Result<Token, Error>>,
{
    pub fn new(lex: L) -> Self {
        Self {
            lex,
            tokens: Vec::new(),
            pos: 0,
            errors: Vec::new(),
        }
    }

    fn ensure_buffer(&mut self) -> bool {
        while self.pos >= self.tokens.len() {
            match self.lex.next() {
                Some(tokens) => {
                    match tokens {
                        Ok(tokens) => self.tokens.push(tokens),
                        Err(error) => self.errors.push(error),
                    }
                    if !self.tokens.is_empty() {
                        return true;
                    }
                }
                None => return false,
            }
        }
        true
    }

    fn advance(&mut self, steps: usize) -> Option<Token> {
        if !self.ensure_buffer() {
            return None;
        }
        let token = Some(self.tokens[self.pos].clone());
        self.pos += steps;
        token
    }

    fn peek(&mut self) -> Option<Token> {
        if self.ensure_buffer() {
            Some(self.tokens[self.pos].clone())
        } else {
            None
        }
    }

    fn dispatch_node(&mut self, token: Token) -> Node {
        match token.kind {
            TokenType::Int(number) => Node::Literal(Value::Integer(number)),
            TokenType::Float(number) => Node::Literal(Value::Float(number)),
            TokenType::Identifier(identifier) => self.parse_function(identifier),
            TokenType::String(string) => Node::Literal(Value::String(Rc::from(string))),
            TokenType::RelativeReference(x, y) => Node::RelativeReference(x, y),
            TokenType::Variable(name) => Node::Variable(name),
            TokenType::DefineFunction => self.parse_define_function(),
            TokenType::If => self.parse_condition(),
            TokenType::Return => Node::Return(match self.advance(1) {
                Some(token) => Box::from(self.dispatch_node(token)),
                None => Box::from(Node::Literal(Value::Nil))
            }),
            _ => todo!("Unimplemented token: {:?}", token),
        }
    }

    fn parse_function(&mut self, token: String) -> Node {
        let operator = Box::new(Node::Symbol(token));
        if let Some(token) = self.advance(1) && token.kind != TokenType::LeftParen {
            todo!("Behind operator need a left paren!");
        }
        let mut arguments = Vec::new();
        while let Some(argument) = self.advance(1) {
            if argument.kind == TokenType::RightParen {
                break;
            }
            arguments.push(self.dispatch_node(argument));
        }
        Node::Apply {operator, arguments}
    }

    fn parse_define_function(&mut self) -> Node {
        let operator = match self.advance(1) {
            Some(token) => match token.kind {
                TokenType::Variable(name) => name,
                _ => todo!("Function need a name!")
            },
            None => todo!("There is one redundant function definition")
        };
        if let Some(token) = self.advance(1) && token.kind != TokenType::LeftParen {
            todo!("Behind operator need a left paren!");
        }
        let mut arguments = Vec::new();
        while let Some(argument) = self.advance(1) {
            if argument.kind == TokenType::RightParen {
                break;
            }
            arguments.push(match argument.kind {
                TokenType::Variable(name) => Node::Assignment(name), // Because in a lexer when it encounters a function argument, it converts it into a variable
                _ => todo!()
            })
        };
        if let Some(token) = self.advance(1) && token.kind != TokenType::LeftBrace {
            todo!("Behind arguments need a left brace!");
        }
        let mut body = Vec::new();
        while let Some(argument) = self.advance(1) {
            if argument.kind == TokenType::RightBrace {
                break;
            }
            body.push(self.parse_pipeline(argument));
        };
        Node::DefineFunction {operator, arguments, body}
    }

    fn parse_condition(&mut self) -> Node {
        if let Some(token) = self.advance(1) && token.kind != TokenType::LeftParen {
            todo!("Behind condition need a left paren!");
        }
        let mut condition = Vec::new();
        while let Some(argument) = self.advance(1) {
            if argument.kind == TokenType::RightParen {
                break;
            }
            condition.push(self.dispatch_node(argument))
        };
        if let Some(token) = self.advance(1) && token.kind != TokenType::LeftBrace {
            todo!("Behind arguments need a left brace!");
        }
        let mut if_body = Vec::new();
        while let Some(argument) = self.advance(1) {
            if argument.kind == TokenType::RightBrace {
                break;
            }
            if_body.push(self.parse_pipeline(argument));
        };
        let mut else_body = Vec::new();
        if let Some(token) = self.advance(1) && token.kind != TokenType::Else {
            return Node::Condition { condition, if_body, else_body }
        }
        if let Some(token) = self.advance(1) && token.kind != TokenType::LeftBrace {
            todo!("Behind arguments need a left brace!");
        }
        while let Some(argument) = self.advance(1) {
            if argument.kind == TokenType::RightBrace {
                break;
            }
            else_body.push(self.parse_pipeline(argument));
        };
        Node::Condition { condition, if_body, else_body }
    }

    fn parse_pipeline(&mut self, token: Token) -> Node {
        let mut stations = Vec::new();
        stations.push(self.dispatch_node(token));
        if self.peek().is_none() || self.peek().unwrap().kind != TokenType::Arrow {
            match stations.pop() {
                Some(station) => return station,
                _ => todo!("There is no station before pipeline!"),
            }
        }
        while let Some(token) = self.peek() && token.kind == TokenType::Arrow {
            self.advance(1);
            if let Some(token) = self.advance(1) {
                stations.push(match self.dispatch_node(token) {
                    Node::Variable(name) => Node::Assignment(name),
                    other => other
                });
            } else {
                todo!("We should make error_handle.rs")
            }
        }
        Node::Pipeline(stations)
    }

    fn error_pusher() {}
}

impl<L> Iterator for Parser<L>
where
    L: Iterator<Item = Result<Token, Error>>,
{
    type Item = Node;
    fn next(&mut self) -> Option<Self::Item> {
        self.advance(1).map(|token| self.parse_pipeline(token))
    }
}

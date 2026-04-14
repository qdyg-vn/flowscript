use crate::error_handler::{Error, SyntaxError, SyntaxErrorType};
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
        }
    }

    fn ensure_buffer(&mut self) -> Result<bool, Error> {
        while self.pos >= self.tokens.len() {
            match self.lex.next() {
                Some(tokens) => {
                    self.tokens.push(tokens?);
                    if !self.tokens.is_empty() {
                        return Ok(true);
                    }
                }
                None => return Ok(false),
            }
        }
        Ok(true)
    }

    fn advance(&mut self, steps: usize) -> Option<Result<Token, Error>> {
        match self.ensure_buffer() {
            Err(error) => return Some(Err(error)),
            Ok(false) => return None,
            Ok(true) => ()
        }
        let token = Some(Ok(self.tokens[self.pos].clone()));
        self.pos += steps;
        token
    }

    fn peek(&mut self) -> Option<Result<Token, Error>> {
        match self.ensure_buffer() {
            Err(error) => Some(Err(error)),
            Ok(false) => None,
            Ok(true) => Some(Ok(self.tokens[self.pos].clone()))
        }
    }

    fn dispatch_node(&mut self, token: Token) -> Result<Node, Error> {
        match token.kind {
            TokenType::Int(number) => Ok(Node::Literal(Value::Integer(number))),
            TokenType::Float(number) => Ok(Node::Literal(Value::Float(number))),
            TokenType::Boolean(boolean) => Ok(Node::Literal(Value::Boolean(boolean))),
            TokenType::Nil => Ok(Node::Literal(Value::Nil)),
            TokenType::Identifier(identifier) => self.parse_function(identifier),
            TokenType::String(string) => Ok(Node::Literal(Value::String(Rc::from(string)))),
            TokenType::RelativeReference(x, y) => Ok(Node::RelativeReference(x, y)),
            TokenType::Variable(name) => Ok(Node::Variable(name)),
            TokenType::DefineFunction => self.parse_define_function(),
            TokenType::If => self.parse_condition(),
            TokenType::Return => Ok(Node::Return(Box::from(match self.advance(1) {
                Some(token) => self.dispatch_node(token?)?,
                None => Node::Literal(Value::Nil)
            }))),
            _ => Err(self.error_pusher(SyntaxErrorType::UnimplementedToken(token)))
        }
    }

    fn parse_function(&mut self, token: String) -> Result<Node, Error> {
        let operator = Box::new(Node::Symbol(token));
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftParen {
            return Err(self.error_pusher(SyntaxErrorType::MissingLeftParen));
        }
        let mut arguments = Vec::new();
        while let Some(argument) = self.advance(1).transpose()? {
            if argument.kind == TokenType::RightParen {
                break;
            }
            arguments.push(self.dispatch_node(argument)?);
        }
        Ok(Node::Apply {operator, arguments})
    }

    fn parse_define_function(&mut self) -> Result<Node, Error> {
        let operator = match self.advance(1).transpose()? {
            Some(token) => match token.kind {
                TokenType::Variable(name) => name,
                _ => return Err(self.error_pusher(SyntaxErrorType::MissingFunctionName))
            },
            None => return Err(self.error_pusher(SyntaxErrorType::RedundantFunctionDefinition))
        };
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftParen {
            return Err(self.error_pusher(SyntaxErrorType::MissingLeftParen))
        }
        let mut arguments = Vec::new();
        while let Some(argument) = self.advance(1).transpose()? && argument.kind != TokenType::RightParen {
            arguments.push(match argument.kind {
                TokenType::Variable(name) => Node::Assignment(name), // Because in a lexer when it encounters a function argument, it converts it into a variable
                _ => todo!()
            })
        };
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftBrace {
            return Err(self.error_pusher(SyntaxErrorType::MissingLeftBrace))
        }
        let mut body = Vec::new();
        if let Some(argument) = self.advance(1).transpose()? && argument.kind != TokenType::RightBrace {
            body.push(self.parse_pipeline(argument)?);
            self.advance(1);
        };
        Ok(Node::DefineFunction {operator, arguments, body})
    }

    fn parse_condition(&mut self) -> Result<Node, Error> {
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftParen {
            return Err(self.error_pusher(SyntaxErrorType::MissingLeftParen))
        }
        let mut condition = Vec::new();
        if let Some(argument) = self.advance(1).transpose()? && argument.kind != TokenType::RightParen {
            condition.push(self.dispatch_node(argument)?);
            self.advance(1);
        };
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftBrace {
            self.error_pusher(SyntaxErrorType::MissingLeftBrace);
        }
        let mut if_body = Vec::new();
        if let Some(argument) = self.advance(1).transpose()? && argument.kind != TokenType::RightBrace {
            if_body.push(self.parse_pipeline(argument)?);
            self.advance(1);
        };
        let mut else_body = Vec::new();
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::Else {
            return Ok(Node::Condition { condition, if_body, else_body })
        }
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftBrace {
            return Err(self.error_pusher(SyntaxErrorType::MissingLeftBrace))
        }
        if let Some(argument) = self.advance(1).transpose()? && argument.kind != TokenType::RightBrace {
            else_body.push(self.parse_pipeline(argument)?);
            self.advance(1);
        };
        Ok(Node::Condition { condition, if_body, else_body })
    }

    fn parse_pipeline(&mut self, token: Token) -> Result<Node, Error> {
        let mut stations = Vec::new();
        stations.push(self.dispatch_node(token)?);
        if self.peek().is_none() || self.peek().transpose()?.unwrap().kind != TokenType::Arrow {
            match stations.pop() {
                Some(station) => return Ok(station),
                _ => return Err(self.error_pusher(SyntaxErrorType::NoStationBeforePipeline)),
            }
        }
        while let Some(token) = self.peek().transpose()? && token.kind == TokenType::Arrow {
            self.advance(1);
            if let Some(token) = self.advance(1).transpose()? {
                stations.push(match self.dispatch_node(token) {
                    Ok(Node::Variable(name)) => Node::Assignment(name),
                    other => other?
                });
            } else {
                return Err(self.error_pusher(SyntaxErrorType::NoStationAfterPipeline))
            }
        }
        Ok(Node::Pipeline(stations))
    }

    fn error_pusher(&mut self, kind: SyntaxErrorType) -> Error {
        Error::SyntaxError(SyntaxError { line: 999, kind })
    }
}

impl<L> Iterator for Parser<L>
where
    L: Iterator<Item = Result<Token, Error>>,
{
    type Item = Result<Node, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.advance(1).map(|token| self.parse_pipeline(token?))
    }
}

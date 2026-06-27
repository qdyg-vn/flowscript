use crate::error_handler::{Error, SyntaxError, SyntaxErrorType};
use crate::node::Node;
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};
use crate::value::{Kind, LightValue, HeavyValue};

pub struct Parser<'source_code> {
    lexer: Lexer<'source_code>,
    tokens: Vec<Token>,
    pos: usize,
}

impl<'source_code> Parser<'source_code> {
    pub fn new(lexer: Lexer<'source_code>) -> Self {
        Self {
            lexer,
            tokens: Vec::new(),
            pos: 0,
        }
    }

    fn ensure_buffer(&mut self) -> Result<bool, Error> {
        while self.pos >= self.tokens.len() {
            match self.lexer.next() {
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

    fn last_index(&self) -> usize {
        for item in self.tokens.iter().rev() {
            if matches!(item, Token {start: _, end: _, kind: _}) { return item.end }
        }
        0
    }

    fn dispatch_node(&mut self, token: Token) -> Result<Node, Error> {
        match token.kind {
            TokenType::Integer(number) => Ok(Node::Literal(LightValue::Integer(number))),
            TokenType::Float(number) => Ok(Node::Literal(LightValue::Float(number))),
            TokenType::Boolean(boolean) => Ok(Node::Literal(LightValue::Boolean(boolean))),
            TokenType::Nil => Ok(Node::Literal(LightValue::Nil)),
            TokenType::Identifier(identifier) => self.parse_function(token.start, identifier),
            TokenType::String(string) => Ok(Node::HeavyLiteral(HeavyValue::String(string))),
            TokenType::RelativeReference(x, y) => Ok(Node::RelativeReference(x, y)),
            TokenType::Variable(name) => Ok(Node::Variable(name)),
            TokenType::DefineFunction => self.parse_define_function(token.start),
            TokenType::If => self.parse_condition(token.start),
            TokenType::Return => Ok(Node::Return(Box::from(match self.advance(1) {
                Some(token) => self.dispatch_node(token?)?,
                None => Node::Literal(LightValue::Nil)
            }))),
            TokenType::LeftBracket => self.parse_array(token.start),
            _ => Err(self.error_pusher(token.start, SyntaxErrorType::UnimplementedToken(token)))
        }
    }

    fn parse_array(&mut self, start: usize) -> Result<Node, Error> {
        let mut arguments = Vec::new();
        loop {
            let Some(argument) = self.advance(1).transpose()? else { return Err(self.error_pusher(start, SyntaxErrorType::MissingRightBracket)) };
            if argument.kind == TokenType::RightBracket { break }
            arguments.push(self.dispatch_node(argument)?);
        }
        Ok(Node::Array(arguments))
    }

    fn parse_function(&mut self, start: usize, operator: String) -> Result<Node, Error> {
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftParen {
            return Err(self.error_pusher(start, SyntaxErrorType::MissingLeftParen));
        }
        let mut arguments = Vec::new();
        loop {
            let Some(argument) = self.advance(1).transpose()? else { return Err(self.error_pusher(start, SyntaxErrorType::MissingRightParen)) };
            if argument.kind == TokenType::RightParen { break }
            arguments.push(self.dispatch_node(argument)?);
        }
        Ok(Node::Apply {operator, arguments})
    }

    fn parse_assignment(&mut self, name: String) -> Result<Node, Error> {
        self.advance(1);
        let Some(token) = self.advance(1).transpose()? else {return Err(self.error_pusher(self.last_index(), SyntaxErrorType::MissingTypeIdentity))};
        match token.kind {
            TokenType::Kind(kind) => {
                let node_kind = match kind.as_str() {
                    "boolean" => Kind::Boolean,
                    "float" => Kind::Float,
                    "integer" => Kind::Integer,
                    "string" => Kind::String,
                    "array" => Kind::Array,
                    _ => unreachable!(),
                };
                Ok(Node::Assignment(name, node_kind))
            },
            _ => Err(self.error_pusher(token.start, SyntaxErrorType::MissingTypeIdentity))
        }
    }

    fn parse_define_function(&mut self, start: usize) -> Result<Node, Error> {
        let operator = match self.advance(1).transpose()? {
            Some(Token {kind: TokenType::Variable(name), ..}) | Some(Token {kind: TokenType::Identifier(name), ..}) => name,
            Some(_) => return Err(self.error_pusher(start, SyntaxErrorType::MissingFunctionName)),
            None => return Err(self.error_pusher(start, SyntaxErrorType::RedundantFunctionDefinition))
        };
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftParen {
            return Err(self.error_pusher(start, SyntaxErrorType::MissingLeftParen))
        }
        let mut arguments = Vec::new();
        loop {
            let Some(argument) = self.advance(1).transpose()? else { return Err(self.error_pusher(self.last_index(), SyntaxErrorType::MissingTypeIdentity)) };
            if argument.kind == TokenType::RightParen { break }
            arguments.push(match argument.kind {
                TokenType::Variable(name) => {
                    if let Some(token) = self.peek().transpose()? && token.kind == TokenType::Colon {
                        self.parse_assignment(name)?
                    } else {
                        Node::SoftAssignment(name) // Because in a lexer when it encounters a function argument, it converts it into a variable
                    }
                },
                _ => todo!()
            })
        };
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftBrace {
            return Err(self.error_pusher(start, SyntaxErrorType::MissingLeftBrace))
        }
        let mut body = Vec::new();
        loop {
            let Some(argument) = self.advance(1).transpose()? else { return Err(self.error_pusher(start, SyntaxErrorType::MissingRightBrace)) };
            if argument.kind == TokenType::RightBrace { break }
            body.push(self.parse_pipeline(argument)?);
        };
        Ok(Node::DefineFunction {operator, arguments, body})
    }

    fn parse_condition(&mut self, start: usize) -> Result<Node, Error> {
        let mut branches: Vec<(Vec<Node>, Vec<Node>)> = Vec::new();
        let mut final_branch = Vec::new();
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftParen {
            return Err(self.error_pusher(start, SyntaxErrorType::MissingLeftParen))
        }
        let mut condition = Vec::new();
        if let Some(argument) = self.advance(1).transpose()? && argument.kind != TokenType::RightParen {
            condition.push(self.dispatch_node(argument)?);
            self.advance(1);
        };
        if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftBrace {
            self.error_pusher(start, SyntaxErrorType::MissingLeftBrace);
        }
        let mut body = Vec::new();
        if let Some(argument) = self.advance(1).transpose()? && argument.kind != TokenType::RightBrace {
            body.push(self.parse_pipeline(argument)?);
            self.advance(1);
        };
        if condition.is_empty() {
            return Err(self.error_pusher(start, SyntaxErrorType::MissingCondition))
        }
        branches.push((condition, body));
        while let Some(token) = self.peek().transpose()? && token.kind == TokenType::Else {
            self.advance(1);
            if let Some(token) = self.peek().transpose()? && token.kind != TokenType::If {
                if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftBrace {
                    return Err(self.error_pusher(start, SyntaxErrorType::MissingLeftBrace))
                }
                if let Some(argument) = self.advance(1).transpose()? && argument.kind != TokenType::RightBrace {
                    final_branch.push(self.parse_pipeline(argument)?);
                    self.advance(1);
                };
                return Ok(Node::Condition { branches, final_branch })
            }
            self.advance(1);
            if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftParen {
                return Err(self.error_pusher(start, SyntaxErrorType::MissingLeftParen))
            }
            let mut condition = Vec::new();
            if let Some(argument) = self.advance(1).transpose()? && argument.kind != TokenType::RightParen {
                condition.push(self.dispatch_node(argument)?);
                self.advance(1);
            };
            if let Some(token) = self.advance(1).transpose()? && token.kind != TokenType::LeftBrace {
                self.error_pusher(start, SyntaxErrorType::MissingLeftBrace);
            }
            let mut body = Vec::new();
            if let Some(argument) = self.advance(1).transpose()? && argument.kind != TokenType::RightBrace {
                body.push(self.parse_pipeline(argument)?);
                self.advance(1);
            };
            if condition.is_empty() {
                return Err(self.error_pusher(start, SyntaxErrorType::MissingCondition))
            }
            branches.push((condition, body))
        }
        Ok(Node::Condition { branches, final_branch })
    }

    fn parse_pipeline(&mut self, token: Token) -> Result<Node, Error> {
        let start = token.start;
        let mut stations = Vec::new();
        stations.push(self.dispatch_node(token)?);
        while let Some(token) = self.peek().transpose()? && token.kind == TokenType::Arrow {
            self.advance(1);
            let Some(token) = self.advance(1).transpose()? else {return Err(self.error_pusher(start, SyntaxErrorType::NoStationAfterPipeline))};
            stations.push(match self.dispatch_node(token) {
                Ok(Node::Variable(name)) => {
                    if let Some(token) = self.peek().transpose()? && token.kind == TokenType::Colon {
                        self.parse_assignment(name)?
                    } else {
                        Node::SoftAssignment(name)
                    }
                },
                other => other?
            });
        }
        Ok(Node::Pipeline(stations))
    }

    fn error_pusher(&mut self, start: usize, kind: SyntaxErrorType) -> Error {
        let (line, column) = self.lexer.find_line_col(start);
        Error::SyntaxError(SyntaxError { line, column, kind })
    }
}

impl<'source_code> Iterator for Parser<'source_code> {
    type Item = Result<Node, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.advance(1).map(|token| self.parse_pipeline(token?))
    }
}

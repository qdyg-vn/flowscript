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

    fn dispatch_node(&mut self, token: Token, errors: &mut Vec<Error>) -> Option<Node> {
        match token.kind {
            TokenType::Integer(number) => Some(Node::Literal(LightValue::Integer(number))),
            TokenType::Float(number) => Some(Node::Literal(LightValue::Float(number))),
            TokenType::Boolean(boolean) => Some(Node::Literal(LightValue::Boolean(boolean))),
            TokenType::Nil => Some(Node::Literal(LightValue::Nil)),
            TokenType::Identifier(identifier) => self.parse_function(token.start, identifier, errors),
            TokenType::String(string) => Some(Node::HeavyLiteral(HeavyValue::String(string))),
            TokenType::RelativeReference(x, y) => Some(Node::RelativeReference(x, y)),
            TokenType::Variable(name) => Some(Node::Variable(name)),
            TokenType::DefineFunction => self.parse_define_function(token.start, errors),
            TokenType::If => self.parse_condition(token.start, errors),
            TokenType::Return => Some(Node::Return(Box::from(match self.advance(1) {
                Some(Ok(token)) => self.dispatch_node(token, errors)?,
                Some(Err(error)) => { errors.push(error); self.jump(errors); return None },
                None => Node::Literal(LightValue::Nil)
            }))),
            TokenType::LeftBracket => self.parse_array(token.start, errors),
            _ => { self.error_pusher(token.start, SyntaxErrorType::UnimplementedToken(token), errors); None }
        }
    }

    fn parse_array(&mut self, start: usize, errors: &mut Vec<Error>) -> Option<Node> {
        let mut arguments = Vec::new();
        loop {
            match self.advance(1) {
                Some(Ok(Token { kind: TokenType::RightBracket, .. })) => break,
                Some(Ok(argument)) => arguments.push(self.dispatch_node(argument, errors)?),
                Some(Err(error)) => errors.push(error),
                None => { self.error_pusher(start, SyntaxErrorType::MissingRightBracket, errors); return None },
            };
        }
        Some(Node::Array(arguments))
    }

    fn parse_function(&mut self, start: usize, operator: String, errors: &mut Vec<Error>) -> Option<Node> {
        self.advance(1);
        let mut arguments = Vec::new();
        loop {
            match self.advance(1) {
                Some(Ok(Token { kind: TokenType::RightParen, .. })) => break,
                Some(Ok(argument)) => arguments.push(self.dispatch_node(argument, errors)?),
                Some(Err(error)) => errors.push(error),
                None => { self.error_pusher(start, SyntaxErrorType::MissingRightParen, errors); return None },
            }
        }
        Some(Node::Apply {operator, arguments})
    }

    fn parse_assignment(&mut self, name: String, errors: &mut Vec<Error>) -> Option<Node> {
        self.advance(1);
        let token = match self.advance(1) {
            Some(Ok(token)) => token,
            Some(Err(error)) => { errors.push(error); return None }
            None => { self.error_pusher(self.last_index(), SyntaxErrorType::MissingTypeIdentity, errors); return None },
        };
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
                Some(Node::Assignment(name, node_kind))
            },
            _ => { self.error_pusher(token.start, SyntaxErrorType::InvalidTypeError, errors); None }
        }
    }

    fn parse_define_function(&mut self, start: usize, errors: &mut Vec<Error>) -> Option<Node> {
        let operator = match self.peek() {
            Some(Ok(Token {kind: TokenType::Identifier(name), ..})) => { self.advance(1); name },
            Some(Ok(_)) => { self.error_pusher(start, SyntaxErrorType::MissingFunctionName, errors); String::default() },
            Some(Err(error)) => { errors.push(error); String::default() }
            None => { self.error_pusher(start, SyntaxErrorType::RedundantFunctionDefinition, errors); return None }
        };
        match self.peek() {
            Some(Ok(token)) => if token.kind != TokenType::LeftParen {
                if token.kind != TokenType::RightParen { self.advance(1); }
                self.error_pusher(start, SyntaxErrorType::MissingLeftParen, errors)
            } else { self.advance(1); },
            Some(Err(error)) => errors.push(error),
            None => self.error_pusher(start, SyntaxErrorType::RedundantFunctionDefinition, errors),
        }
        let mut arguments = Vec::new();
        loop {
            match self.advance(1) {
                Some(Ok(Token { kind: TokenType::RightParen, .. })) => break,
                Some(Ok(argument)) => arguments.push(match argument.kind {
                    TokenType::Variable(name) => {
                        if let Some(Ok(token)) = self.peek() && token.kind == TokenType::Colon {
                            self.parse_assignment(name, errors)?
                        } else {
                            Node::SoftAssignment(name) // Because in a lexer when it encounters a function argument, it converts it into a variable
                        }
                    },
                    _ => todo!()
                }),
                Some(Err(error)) => errors.push(error),
                None => { self.error_pusher(self.last_index(), SyntaxErrorType::MissingTypeIdentity, errors); return None }
            }
        };
        match self.peek() {
            Some(Ok(token)) => if token.kind != TokenType::LeftBrace {
                if token.kind != TokenType::RightBrace { self.advance(1); }
                self.error_pusher(start, SyntaxErrorType::MissingLeftBrace, errors)
            } else { self.advance(1); },
            Some(Err(error)) => errors.push(error),
            None => self.error_pusher(start, SyntaxErrorType::MissingFunctionBody, errors),
        }
        let mut body = Vec::new();
        loop {
            match self.advance(1) {
                Some(Ok(Token { kind: TokenType::RightBrace, .. })) => break,
                Some(Ok(argument)) => match self.parse_pipeline(argument) {
                    Ok(pipeline) => body.push(pipeline),
                    Err(body_errors) => errors.extend(body_errors)
                },
                Some(Err(error)) => errors.push(error),
                None => { self.error_pusher(start, SyntaxErrorType::MissingRightBrace, errors); return None },
            };
        };
        Some(Node::DefineFunction {operator, arguments, body})
    }

    fn parse_condition_expression(&mut self, start: usize, errors: &mut Vec<Error>) -> Vec<Node> {
        match self.peek() {
            Some(Ok(token)) => if token.kind != TokenType::LeftParen {
                if token.kind != TokenType::RightParen { self.advance(1); }
                self.error_pusher(start, SyntaxErrorType::MissingLeftParen, errors)
            } else { self.advance(1); },
            Some(Err(error)) => errors.push(error),
            None => self.error_pusher(start, SyntaxErrorType::RedundantCondition, errors),
        }
        let mut condition = Vec::new();
        match self.peek() {
            Some(Ok(argument)) => if argument.kind != TokenType::RightParen {
                self.advance(1);
                if let Some(node) = self.dispatch_node(argument, errors) {
                    condition.push(node)
                }
                self.advance(1);
            } else { self.advance(1); },
            Some(Err(error)) => errors.push(error),
            None => self.error_pusher(start, SyntaxErrorType::MissingRightParen, errors),
        }
        condition
    }

    fn parse_condition_body(&mut self, start: usize, errors: &mut Vec<Error>) -> Vec<Node> {
        match self.peek() {
            Some(Ok(token)) => if token.kind != TokenType::LeftBrace {
                if token.kind != TokenType::RightBrace { self.advance(1); }
                self.error_pusher(start, SyntaxErrorType::MissingLeftBrace, errors)
            } else { self.advance(1); },
            Some(Err(error)) => errors.push(error),
            None => self.error_pusher(start, SyntaxErrorType::MissingConditionBody, errors),
        };
        let mut body = Vec::new();
        match self.peek() {
            Some(Ok(argument)) => if argument.kind != TokenType::RightBrace {
                self.advance(1);
                match self.parse_pipeline(argument) {
                    Ok(pipeline) => body.push(pipeline),
                    Err(body_errors) => errors.extend(body_errors)
                }
                self.advance(1);
            } else { self.advance(1); },
            Some(Err(error)) => errors.push(error),
            None => self.error_pusher(start, SyntaxErrorType::MissingConditionBody, errors),
        };
        body
    }

    fn parse_condition(&mut self, start: usize, errors: &mut Vec<Error>) -> Option<Node> {
        let mut branches: Vec<(Vec<Node>, Vec<Node>)> = Vec::new();
        let condition = self.parse_condition_expression(start, errors);
        let body = self.parse_condition_body(start, errors);
        if condition.is_empty() {
            self.error_pusher(start, SyntaxErrorType::MissingCondition, errors)
        }
        branches.push((condition, body));
        while let Some(result) = self.peek() {
            match result {
                Ok(Token { kind: TokenType::Else, .. }) => {}
                Ok(_) => break,
                Err(error) => errors.push(error)
            }
            self.advance(1);
            match self.peek() {
                Some(Ok(Token { kind: TokenType::If, .. })) => {}
                Some(Ok(_)) => {
                    let final_branch = self.parse_condition_body(start, errors);
                    return Some(Node::Condition { branches, final_branch })
                }
                Some(Err(error)) => errors.push(error),
                None => self.error_pusher(start, SyntaxErrorType::RedundantCondition, errors)
            }
            self.advance(1);
            let condition = self.parse_condition_expression(start, errors);
            let body = self.parse_condition_body(start, errors);
            if condition.is_empty() {
                self.error_pusher(start, SyntaxErrorType::MissingCondition, errors)
            }
            branches.push((condition, body));
        }
        Some(Node::Condition { branches, final_branch: vec![] })
    }

    fn parse_pipeline(&mut self, token: Token) -> Result<Node, Vec<Error>> {
        let start = token.start;
        let mut stations = Vec::new();
        let mut errors = Vec::new();
        if let Some(node) = self.dispatch_node(token, &mut errors) {
            stations.push(node)
        }
        while let Some(result) = self.peek() {
            match result {
                Ok(Token { kind: TokenType::Arrow, .. }) => {},
                Ok(_) => break,
                Err(error) => { errors.push(error); continue }
            };
            self.advance(1);
            let token = match self.advance(1) {
                Some(Ok(token)) => token,
                Some(Err(error)) => { errors.push(error); continue },
                None => { self.error_pusher(start, SyntaxErrorType::NoStationAfterPipeline, &mut errors); break }
            };
            match self.dispatch_node(token, &mut errors) {
                Some(Node::Variable(name)) => {
                    match self.peek() {
                        Some(Ok(token)) => if token.kind == TokenType::Colon {
                            let Some(node) = self.parse_assignment(name, &mut errors) else { continue };
                            stations.push(node)
                        },
                        Some(Err(error)) => { errors.push(error); continue }
                        None => stations.push(Node::SoftAssignment(name))
                    }
                },
                Some(other) => stations.push(other),
                None => continue,
            };
            self.advance(1);
        }
        if errors.is_empty() {
            Ok(Node::Pipeline(stations))
        } else {
            Err(errors)
        }
    }

    fn error_pusher(&mut self, start: usize, kind: SyntaxErrorType, errors: &mut Vec<Error>) {
        let (line, column) = self.lexer.find_line_col(start);
        errors.push(Error::SyntaxError(SyntaxError { line, column, kind }))
    }

    fn jump(&mut self, errors: &mut Vec<Error>) {
        while let Some(token) = self.peek() {
            match token {
                Ok(token) => if matches!(token.kind, TokenType::Arrow) { break },
                Err(error) => errors.push(error)
            }
            self.advance(1);
        }
    }
}

impl<'source_code> Iterator for Parser<'source_code> {
    type Item = Result<Node, Vec<Error>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.advance(1) {
            Some(Ok(token)) => Some(self.parse_pipeline(token)),
            Some(Err(error)) => Some(Err(vec![error])),
            None => None
        }
    }
}

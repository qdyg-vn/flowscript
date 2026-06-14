use crate::node::Node;
use crate::error_handler::{ErrorHandler, Error};

pub struct Optimizer<P>
where
    P: Iterator<Item = Result<Node, Error>>,
{
    parser: P,
    error_handler: ErrorHandler,
}

impl<P> Optimizer<P>
where
    P: Iterator<Item = Result<Node, Error>>,
{
    pub fn new(parser: P, error_handler: ErrorHandler) -> Self {
        Self { parser, error_handler }
    }

    pub fn optimize(mut self) -> (Vec<Node>, ErrorHandler) {
        let mut nodes = Vec::new();
        for item in self.parser.by_ref() {
            match item {
                Ok(node) => nodes.push(node),
                Err(error) => self.error_handler.errors.push(error)
            }
        }
        if !self.error_handler.errors.is_empty() { self.error_handler.report_exit() }
        (nodes, self.error_handler)
    }
}

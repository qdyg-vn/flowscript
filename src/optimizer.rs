use crate::node::Node;

pub struct Optimizer<P>
where
    P: Iterator<Item = Node>,
{
    parser: P,
}

impl<P> Optimizer<P>
where
    P: Iterator<Item = Node>,
{
    pub fn new(parser: P) -> Self {
        Self { parser }
    }

    pub fn optimize(&mut self) -> Vec<Node> {
        let ast = (&mut self.parser).collect();
        ast
    }
}

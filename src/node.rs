struct Node {
    pub ty: NodeType,
    pub children: Vec<Node>,
    pub operand: String,
}

#[derive(Debug, PartialEq, Eq)]
enum NodeType {
    CALL = 1,
    LOAD = 2,
    IDENTIFIER = 3,
    FUNCTION = 4,
}
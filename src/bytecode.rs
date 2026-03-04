struct ByteCode {
    pub ty: BytecodeType,
    pub children: Vec<ByteCode>,
    pub operand: String,
}

#[derive(Debug, PartialEq, Eq)]
enum BytecodeType {
    CALL = 1,
    LOAD = 2,
    IDENTIFIER = 3,
    FUNCTION = 4,
}
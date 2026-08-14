use crate::constants_pool::ConstantsPool;
use crate::instructions::{Chunk, Instruction};
use crate::node::{ConditionBranch, TypedAST, TypedNode};

pub struct Emitter {
    constants_pool: ConstantsPool,
}

impl Emitter {
    pub fn new(constants_pool: ConstantsPool) -> Self {
        Self {
            constants_pool,
        }
    }

    pub fn emit(mut self, ast: TypedAST) -> (ConstantsPool, Vec<Chunk>) {
        let mut map: Vec<Chunk> = Vec::new();
        for instructions in ast.nodes {
            let TypedNode::Pipeline(stations) = instructions else { unreachable!() };
            let mut chunk = Chunk::default();
            self.create_chunk(stations, &mut chunk);
            map.push(chunk)
        }
        (self.constants_pool, map)
    }

    fn create_chunk(&mut self, stations: Vec<TypedNode>, chunk: &mut Chunk) {
        for station in stations {
            match station {
                TypedNode::Literal(value) => {
                    let index = self.constants_pool.add_constant(value);
                    chunk.instructions.push(Instruction::Load(index as u32))
                },
                TypedNode::HeavyLiteral(heavy_value) => {
                    let index = self.constants_pool.add_heavy_constant(&heavy_value);
                    chunk.instructions.push(Instruction::Load(index as u32))
                },
                TypedNode::BuiltinCall {index, arguments, ..} => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::BuiltinCall(index, arity))
                },
                TypedNode::Call { signature_index, arguments, ..} => {
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::Call(signature_index as u16))
                },
                TypedNode::RelativeReference(x, y, _) => chunk.instructions.push(Instruction::RelativeReference(x, y)),
                TypedNode::Assignment(index, kind) => chunk.instructions.push(Instruction::Store(index, kind as u8)),
                TypedNode::Variable(index, _) => chunk.instructions.push(Instruction::LoadVariable(index)),
                TypedNode::DefineFunction { signature_index, body } => {
                    let mut pipelines = Vec::with_capacity(body.nodes.len());
                    for node in body.nodes {
                        let TypedNode::Pipeline(pipeline) = node else { unreachable!() };
                        let mut child_chunk = Chunk { instructions: Vec::with_capacity(pipeline.len()), arity: body.arity, variables_count: body.variables_count };
                        self.create_chunk(pipeline, &mut child_chunk);
                        pipelines.push(child_chunk)
                    }
                    self.constants_pool.write_function_body(signature_index as usize, pipelines);
                },
                TypedNode::Pipeline(stations) => self.create_chunk(stations, chunk),
                TypedNode::Condition {branches, final_branch} => self.emit_condition(branches, final_branch, chunk),
                TypedNode::Return(value) => {
                    self.create_chunk(vec![*value], chunk);
                    chunk.instructions.push(Instruction::Return)
                },
                TypedNode::Array(elements) => {
                    let count = elements.len() as u32;
                    self.create_chunk(elements, chunk);
                    chunk.instructions.push(Instruction::Array(count))
                },
                TypedNode::Add(arguments, kind, _) => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::Add(arity, kind))
                }
                TypedNode::Minus(arguments, kind, _) => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::Minus(arity, kind))
                }
                TypedNode::Multiply(arguments, kind, _) => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::Multiply(arity, kind))
                }
                TypedNode::Equal(arguments, kind, _) => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::Equal(arity, kind))
                },
                TypedNode::LessThan(arguments, kind, _) => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::LessThan(arity, kind))
                },
                TypedNode::GreaterThan(arguments, kind, _) => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::Reverse(arity));
                    chunk.instructions.push(Instruction::LessThan(arity, kind))
                },
                TypedNode::LessThanOrEqual(arguments, kind, _) => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::Reverse(arity));
                    chunk.instructions.push(Instruction::LessThan(arity, kind));
                    chunk.instructions.push(Instruction::Not);
                },
                TypedNode::GreaterThanOrEqual(arguments, kind, _) => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::LessThan(arity, kind));
                    chunk.instructions.push(Instruction::Not);
                },
                TypedNode::NotEqual(arguments, kind, _) => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::Equal(arity, kind));
                    chunk.instructions.push(Instruction::Not);
                }
            }
        }
    }

    fn emit_condition(&mut self, branches: Vec<ConditionBranch<TypedNode>>, final_branch: Vec<TypedNode>, chunk: &mut Chunk) {
        let mut complete_positions = Vec::with_capacity(branches.len());
        for branch in branches {
            self.create_chunk(branch.condition, chunk);
            let branch_position = chunk.instructions.len();
            chunk.instructions.push(Instruction::JumpIfFalse(0));
            self.create_chunk(branch.body, chunk);
            complete_positions.push(chunk.instructions.len());
            chunk.instructions.push(Instruction::Jump(0));
            let next_branch = chunk.instructions.len();
            if let Instruction::JumpIfFalse(target) = &mut chunk.instructions[branch_position] { *target = next_branch as u16 };
        }
        self.create_chunk(final_branch, chunk);
        let end_position = chunk.instructions.len();
        for index in complete_positions {
            if let Instruction::Jump(target) = &mut chunk.instructions[index] { *target = end_position as u16 };
        }
    }
}

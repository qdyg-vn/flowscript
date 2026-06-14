use crate::constants_pool::ConstantsPool;
use crate::instructions::{Chunk, Instruction};
use crate::node::{ResolvedNode, AST};
use crate::value::HeavyValue;

pub struct Emitter {
    constants_pool: ConstantsPool,
}

impl Emitter {
    pub fn new(constants_pool: ConstantsPool) -> Self {
        Self {
            constants_pool,
        }
    }

    pub fn emit(mut self, ast: AST) -> (ConstantsPool, Vec<Chunk>) {
        let mut map: Vec<Chunk> = Vec::new();
        for instructions in ast.nodes {
            if let ResolvedNode::Pipeline(stations) = instructions {
                let mut chunk = Chunk { instructions: Vec::new(), arity: 0 };
                self.create_chunk(stations, &mut chunk);
                map.push(chunk)
            }
        }
        (self.constants_pool, map)
    }

    fn create_chunk(&mut self, stations: Vec<ResolvedNode>, chunk: &mut Chunk) {
        for station in stations {
            match station {
                ResolvedNode::Literal(value) => {
                    let index = self.constants_pool.add_constant(value);
                    chunk.instructions.push(Instruction::Load(index as u32))
                },
                ResolvedNode::HeavyLiteral(heavy_value) => {
                    let index = self.constants_pool.add_heavy_constant(heavy_value);
                    chunk.instructions.push(Instruction::Load(index as u32))
                },
                ResolvedNode::BuiltinCall {index, arguments} => {
                    let arity = arguments.len() as u16;
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::BuiltinCall(index, arity))
                },
                ResolvedNode::Call {scope, index, arguments} => {
                    self.create_chunk(arguments, chunk);
                    chunk.instructions.push(Instruction::Call(scope, index))
                },
                ResolvedNode::RelativeReference(x, y) => chunk.instructions.push(Instruction::RelativeReference(x, y)),
                ResolvedNode::Assignment(index) => chunk.instructions.push(Instruction::Store(index)),
                ResolvedNode::HardAssignment(index, kind) => chunk.instructions.push(Instruction::HardStore(index, kind as u8)),
                ResolvedNode::Variable(index) => chunk.instructions.push(Instruction::LoadVariable(index)),
                ResolvedNode::DefineFunction {index, body} => {
                    let mut child_chunk = Chunk { instructions: Vec::new(), arity: body.arity };
                    self.create_chunk(body.nodes, &mut child_chunk);
                    let body_index = self.constants_pool.add_heavy_constant(HeavyValue::Function(child_chunk));
                    chunk.instructions.push(Instruction::DefineFunction(index, body_index as u16));
                },
                ResolvedNode::Pipeline(stations) => self.create_chunk(stations, chunk),
                ResolvedNode::Condition {branches, final_branch} => self.emit_condition(branches, final_branch, chunk),
                ResolvedNode::Return(value) => {
                    self.create_chunk(vec![*value], chunk);
                    chunk.instructions.push(Instruction::Return)
                },
                ResolvedNode::Array(elements) => {
                    let count = elements.len() as u32;
                    self.create_chunk(elements, chunk);
                    chunk.instructions.push(Instruction::Array(count))
                },
            }
        }
    }

    fn emit_condition(&mut self, branches: Vec<(Vec<ResolvedNode>, Vec<ResolvedNode>)>, final_branch: Vec<ResolvedNode>, chunk: &mut Chunk) {
        let mut complete_positions = Vec::with_capacity(branches.len());
        for (condition, body) in branches {
            self.create_chunk(condition, chunk);
            let branch_position = chunk.instructions.len();
            chunk.instructions.push(Instruction::JumpIfFalse(0));
            self.create_chunk(body, chunk);
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
use common::chunk::Chunk;

pub mod compiler;
pub mod scanner;

pub fn compile(code: &str) -> Option<Chunk> {
    compiler::compile(code)
}

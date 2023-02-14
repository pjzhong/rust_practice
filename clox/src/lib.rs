mod chunk;
mod vm;

pub type Value = f64;
pub use chunk::{Chunk, OpCode};
pub use vm::Vm;

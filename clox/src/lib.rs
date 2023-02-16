mod chunk;
mod front;
mod vm;

pub type Value = f64;
pub use chunk::{Chunk, OpCode};
pub use vm::{interpret, InterpretResult, Vm};

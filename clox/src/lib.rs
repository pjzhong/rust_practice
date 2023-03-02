mod chunk;
mod front;
mod value;
mod vm;

use chunk::{Chunk, OpCode};
use value::{Value};
pub use vm::{interpret, InterpretResult, Vm};

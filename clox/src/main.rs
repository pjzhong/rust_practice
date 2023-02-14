use clox::{Chunk, OpCode, Vm};

fn main() {

    let mut chunk = Chunk::default();
    let const_idx = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant.into(), 123);
    chunk.write(const_idx as u8, 123);
    chunk.write(OpCode::Return.into(), 123);

    let mut  vm = Vm::new();

    vm.interpret(chunk);
}

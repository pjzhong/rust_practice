use clox::{Chunk, OpCode, Vm};

fn main() {
    let mut chunk = Chunk::default();
    let const_idx = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant, 123);
    chunk.write(const_idx as u8, 123);

    let const_idx = chunk.add_constant(3.4);
    chunk.write(OpCode::Constant, 123);
    chunk.write(const_idx as u8, 123);

    chunk.write(OpCode::Add, 123);

    let const_idx = chunk.add_constant(5.6);
    chunk.write(OpCode::Constant, 123);
    chunk.write(const_idx as u8, 123);

    chunk.write(OpCode::Divide, 123);
    chunk.write(OpCode::Negate, 123);
    chunk.write(OpCode::Return, 123);

    let mut vm = Vm::new();

    vm.interpret(chunk);
}

use crate::{
    chunk::{Chunk, OpCode},
    Value,
};

#[derive(Default)]
pub struct Vm {
    chunk: Chunk,
    chunk_idx: usize,
    ip: usize,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::default(),
            chunk_idx: 0,
            ip: 0,
        }
    }

    pub fn free() {}

    pub fn interpret(&mut self, chunks: Chunk) -> InterpretResult {
        self.chunk = chunks;
        self.ip = 0;
        loop {
            #[cfg(debug_assertions)]
            self.chunk.disassemble_instruction(self.ip);

            let inst = self.read_byte().into();
            match inst {
                OpCode::Return => return InterpretResult::Ok,
                OpCode::Constant => {
                    let consant = self.read_consnt();
                    print!("{:?}", consant);
                    println!();
                }
                OpCode::Unknown(a) => {
                    eprintln!(
                        "chunk_idx:{:?}, ip:{:?}, byte:{:?}",
                        self.chunk_idx, self.ip, a
                    )
                }
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let res = self.chunk.read_byte(self.ip);
        self.ip += 1;
        res
    }

    fn read_consnt(&mut self) -> Value {
        let idx = self.read_byte();
        self.chunk.read_constant(idx as usize)
    }
}

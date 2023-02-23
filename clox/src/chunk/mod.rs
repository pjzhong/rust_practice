pub use crate::chunk::op::OpCode;
use crate::Value;

mod op;

#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<u32>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn line(&self, offset: usize) -> Option<u32> {
        self.lines.get(offset).copied()
    }

    pub fn read_byte(&self, offset: usize) -> u8 {
        self.code[offset]
    }

    pub fn read_constant(&self, offset: usize) -> Value {
        self.constants[offset].clone()
    }

    pub fn write(&mut self, byte: impl Into<u8>, line: u32) {
        self.code.push(byte.into());
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble_chunk(&self, name: &str) {
        println!("== {} ==", name);

        let len = self.code.len();
        let mut offset = 0;
        while offset < len {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:04} ", self.lines[offset])
        }

        let instruction = self.code[offset].into();
        match instruction {
            OpCode::Constant | OpCode::DefineGlobal | OpCode::GetGlobal => {
                self.constant_instruction(instruction, offset)
            }
            OpCode::Unknown(inst) => {
                println!("Unknow opcde {}", inst);
                offset + 1
            }
            _ => self.simple_instruction(&instruction, offset),
        }
    }

    fn simple_instruction(&self, name: &OpCode, offset: usize) -> usize {
        println!("{:?}", name);
        offset + 1
    }

    fn constant_instruction(&self, name: OpCode, offset: usize) -> usize {
        let const_idx = self.code[offset + 1];
        print!("{:-16} {:04} ", format!("{:?}", name), const_idx);
        println!("{:?}", &self.constants[const_idx as usize]);
        offset + 2
    }
}

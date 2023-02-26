pub use crate::chunk::op::OpCode;
use crate::Value;
use std::ops::{Add, Sub};

mod op;

#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<u32>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn code(&mut self) -> &mut [u8] {
        &mut self.code
    }

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
        for (idx, val) in self.constants.iter().enumerate() {
            if val == &value {
                return idx;
            }
        }
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
            OpCode::Constant | OpCode::DefineGlobal | OpCode::GetGlobal | OpCode::SetGlobal => {
                self.constant_instruction(instruction, offset)
            }
            OpCode::GetLocal | OpCode::SetLocal => self.byte_instruction(instruction, offset),
            OpCode::JumpIfFalse | OpCode::JumpIfTrue | OpCode::Jump => {
                self.jump_instruction(instruction, usize::add, offset)
            }
            OpCode::Loop => {
                self.jump_instruction(instruction, usize::sub, offset)
            }
            OpCode::Unknown(inst) => {
                println!("Unknow opcde {}", inst);
                offset + 1
            }
            OpCode::Return
            | OpCode::Negate
            | OpCode::Add
            | OpCode::Subtract
            | OpCode::Multiply
            | OpCode::Divide
            | OpCode::Nil
            | OpCode::True
            | OpCode::False
            | OpCode::Bang
            | OpCode::Equal
            | OpCode::Greater
            | OpCode::Less
            | OpCode::Print
            | OpCode::Pop => self.simple_instruction(&instruction, offset),
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

    fn byte_instruction(&self, name: OpCode, offset: usize) -> usize {
        let const_idx = self.code[offset + 1];
        println!("{:-16} {:04} ", format!("{:?}", name), const_idx);
        offset + 2
    }

    fn jump_instruction(
        &self,
        name: OpCode,
        op: fn(usize, usize) -> usize,
        offset: usize,
    ) -> usize {
        let jump = {
            let first = (self.code[offset + 1] as u16) << 8;
            let second = (self.code[offset + 2]) as u16;
            (first | second) as usize
        };
        println!(
            "{:-16} {:04} -> {:?}",
            format!("{:?}", name),
            offset,
            op(offset + 3, jump)
        );
        offset + 3
    }
}

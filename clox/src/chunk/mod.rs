use crate::chunk::op::OpCode;

mod op;

pub type Value = f64;

struct Chunk {
    code: Vec<u8>,
    lines: Vec<u32>,
    constants: Vec<Value>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            code: vec![],
            lines: vec![],
            constants: vec![],
        }
    }
}

impl Chunk {
    fn write(&mut self, byte: u8, line: i32) {
        self.code.push(byte)
    }

    fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    fn disassemble_chunk(&self, name: &str) {
        println!("== {} ==", name);

        let len = self.code.len();
        let mut offset = 0;
        while offset < len {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);
        if self.lines[offset] == self.lines[offset - 1] {
            print!("    | ");
        } else {
            print!("{:04}", self.lines[offset])
        }

        let instruction = self.code[offset].into();
        return match instruction {
            OpCode::Return => self.simple_instruction("OP_RETURN", offset),
            OpCode::Constant => self.constant_instruction("OP_CONSTANT", offset),
            OpCode::Unknown(inst) => {
                println!("Unknow opcde {}", inst);
                offset + 1
            }
        };
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let const_idx = self.code[offset + 1];
        print!("{:-16} {:04} ", name, const_idx);
        print_value(&self.constants[const_idx as usize]);
        print!("\n");

        offset + 2
    }
}

fn print_value(value: &Value) {
    print!("{:?}", value)
}

#[cfg(test)]
mod test {

    #[test]
    fn print_chunk() {
        use super::*;

        let mut chunk = Chunk::default();
        let const_idx = chunk.add_constant(1.2);
        chunk.write(OpCode::Constant.into(), 123);
        chunk.write(const_idx as u8, 123);
        chunk.write(OpCode::Return.into(), 123);

        chunk.disassemble_chunk("test chunk");
    }
}

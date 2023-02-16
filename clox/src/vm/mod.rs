use std::collections::VecDeque;

use crate::{
    chunk::{Chunk, OpCode},
    front::Compiler,
    Value,
};

#[derive(Default)]
pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: VecDeque<Value>,
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
            ip: 0,
            stack: VecDeque::new(),
        }
    }

    pub fn free() {}

    pub fn interpret(&mut self, chunks: Chunk) -> InterpretResult {
        self.chunk = chunks;
        self.ip = 0;
        loop {
            #[cfg(debug_assertions)]
            {
                print!("    ");
                for val in &self.stack {
                    print!("[ {:?} ]", val);
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }

            let inst = self.read_byte().into();
            match inst {
                OpCode::Return => {
                    if let Some(val) = self.pop() {
                        println!("{:?}", val);
                    }
                    return InterpretResult::Ok;
                }
                OpCode::Constant => {
                    let constant = self.read_consnt();
                    self.push(constant);
                }
                OpCode::Negate => {
                    if let Some(val) = self.pop() {
                        self.push(-val);
                    }
                }
                OpCode::Add => {
                    if let (Some(b), Some(a)) = (self.pop(), self.pop()) {
                        self.push(a + b);
                    } else {
                        todo!("binary op error handle")
                    }
                }
                OpCode::Subtract => {
                    if let (Some(b), Some(a)) = (self.pop(), self.pop()) {
                        self.push(a - b);
                    } else {
                        todo!("binary op error handle")
                    }
                }
                OpCode::Multiply => {
                    if let (Some(b), Some(a)) = (self.pop(), self.pop()) {
                        self.push(a * b);
                    } else {
                        todo!("binary op error handle")
                    }
                }
                OpCode::Divide => {
                    if let (Some(b), Some(a)) = (self.pop(), self.pop()) {
                        self.push(a / b);
                    } else {
                        todo!("binary op error handle")
                    }
                }
                OpCode::Unknown(a) => {
                    eprintln!("ip:{:?}, byte:{:?}", self.ip, a)
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

    fn push(&mut self, value: Value) {
        self.stack.push_back(value);
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop_back()
    }
}

pub fn interpret(source: &str) -> InterpretResult {
    let compile = Compiler;
    compile.compile(source);

    InterpretResult::Ok
}

use std::ops::{Div, Mul, Sub};

use crate::{
    chunk::{Chunk, OpCode},
    front::Compiler,
    value::{Object, Value},
};

#[derive(Default)]
pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

#[derive(PartialEq, Eq)]
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
            stack: Vec::new(),
        }
    }

    pub fn free() {}

    pub fn run(&mut self, chunks: Chunk) -> InterpretResult {
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
                    return InterpretResult::Ok;
                }
                OpCode::Constant => {
                    let constant = self.read_consnt();
                    self.push(constant);
                }
                OpCode::Negate => {
                    if let Some(Value::Number(val)) = self.pop() {
                        self.push(-val);
                    } else {
                        self.runtime_error("Operand must be a number");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Add => match (self.peak(0), self.peak(1)) {
                    (Some(Value::Number(b)), Some(Value::Number(a))) => {
                        let res = a + b;
                        self.pop();
                        self.pop();
                        self.push(res)
                    }
                    (Some(Value::Obj(Object::Str(b))), Some(Value::Obj(Object::Str(a)))) => {
                        let new_str = {
                            let mut a = a.as_ref().clone();
                            a.push_str(b);
                            a
                        };
                        self.pop();
                        self.pop();
                        self.push(new_str)
                    }
                    _ => {
                        self.runtime_error("Operands must be two numbers or two strings.");
                        return InterpretResult::RuntimeError;
                    }
                },
                OpCode::Subtract => {
                    let res = self.binary_op(f64::sub);
                    if res != InterpretResult::Ok {
                        return res;
                    }
                }
                OpCode::Multiply => {
                    let res = self.binary_op(f64::mul);
                    if res != InterpretResult::Ok {
                        return res;
                    }
                }
                OpCode::Divide => {
                    let res = self.binary_op(f64::div);
                    if res != InterpretResult::Ok {
                        return res;
                    }
                }
                OpCode::Greater => {
                    let res = self.binary_op(|a, b| a > b);
                    if res != InterpretResult::Ok {
                        return res;
                    }
                }
                OpCode::Less => {
                    let res = self.binary_op(|a, b| a < b);
                    if res != InterpretResult::Ok {
                        return res;
                    }
                }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(true),
                OpCode::False => self.push(false),
                OpCode::Equal => {
                    if let (Some(b), Some(a)) = (self.pop(), self.pop()) {
                        self.push(a == b);
                    } else {
                        self.runtime_error("equal must have two operands");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Bang => {
                    let val = self.pop();
                    self.push(is_falsely(&val));
                }
                OpCode::Print => {
                    if let Some(val) = self.pop() {
                        println!("{}", val);
                    }
                }
                OpCode::Pop => {
                    self.pop();
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

    fn push(&mut self, value: impl Into<Value>) {
        self.stack.push(value.into());
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    fn peak(&self, distance: usize) -> Option<&Value> {
        self.stack.get(self.stack.len().wrapping_sub(1 + distance))
    }

    fn binary_op<R>(&mut self, op: fn(f64, f64) -> R) -> InterpretResult
    where
        R: Into<Value>,
    {
        if let (Some(Value::Number(b)), Some(Value::Number(a))) = (self.peak(0), self.peak(1)) {
            let res = op(*a, *b);
            self.pop();
            self.pop();
            self.push(res);
            InterpretResult::Ok
        } else {
            self.runtime_error("Operand must be numbers.");
            InterpretResult::RuntimeError
        }
    }

    fn runtime_error(&self, messgae: &str) {
        eprintln!("{}", messgae);
        eprintln!("line {:?} in script", self.chunk.line(self.ip));
    }
}

pub fn interpret(source: &str, vm: &mut Vm) -> InterpretResult {
    let chunk = Chunk::default();
    let compile = Compiler::default();
    if let Some(chunk) = compile.compile(source, chunk) {
        vm.run(chunk)
    } else {
        InterpretResult::CompileError
    }
}

fn is_falsely(value: &Option<Value>) -> bool {
    match value {
        None => true,
        Some(Value::Nil) => true,
        Some(Value::Bool(bool)) => !bool,
        _ => false,
    }
}

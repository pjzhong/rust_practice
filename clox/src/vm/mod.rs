use std::collections::{HashMap, VecDeque};
use std::ops::{Div, Mul, Sub};
use std::rc::Rc;

use crate::{
    chunk::{Chunk, OpCode},
    front::Compiler,
    value::{Object, Value},
};

#[derive(Default)]
pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: VecDeque<Value>,
    globals: HashMap<Rc<String>, Value>,
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
            stack: VecDeque::new(),
            globals: HashMap::new(),
        }
    }

    pub fn free() {}

    pub fn run(&mut self, chunks: Chunk) -> InterpretResult {
        self.chunk = chunks;
        self.ip = 0;
        self.stack.clear();
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
                    self.push(is_falsely(val.as_ref()));
                }
                OpCode::Print => {
                    if let Some(val) = self.pop() {
                        println!("{}", val);
                    }
                }
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::DefineGlobal => {
                    if let Value::Obj(Object::Str(name)) = self.read_consnt() {
                        if let Some(value) = self.pop() {
                            self.globals.insert(name.clone(), value);
                        } else {
                            self.runtime_error(&format!(
                                "value not exists, define global:{} error",
                                name
                            ));
                            return InterpretResult::RuntimeError;
                        }
                    } else {
                        self.runtime_error("variable name must be a string");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::GetGlobal => {
                    if let Value::Obj(Object::Str(name)) = self.read_consnt() {
                        if let Some(value) = self.globals.get(&name) {
                            self.push(value.clone())
                        } else {
                            self.runtime_error(&format!("Undefined varaible {}", name));
                            return InterpretResult::RuntimeError;
                        }
                    } else {
                        self.runtime_error("variable name must be a string");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::SetGlobal => {
                    if let Value::Obj(Object::Str(name)) = self.read_consnt() {
                        if !self.globals.contains_key(&name) {
                            self.runtime_error(&format!("Undefined varaible {}", name));
                            return InterpretResult::RuntimeError;
                        }
                        let val = self.peak(0).unwrap_or(&Value::Nil);
                        self.globals.insert(name, val.clone());
                    } else {
                        self.runtime_error("variable name must be a string");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte();
                    if let Some(val) = self.stack.get(slot as usize) {
                        self.push(val.clone())
                    } else {
                        self.runtime_error("getLocal operand error");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte();
                    let val = if let Some(val) = self.peak(0) {
                        val.clone()
                    } else {
                        self.runtime_error("setLocal no operand");
                        return InterpretResult::RuntimeError;
                    };

                    if let Some(local) = self.stack.get_mut(slot as usize) {
                        *local = val.clone();
                    } else {
                        self.runtime_error("setLocal target not exits");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_short() as usize;
                    if is_falsely(self.peak(0)) {
                        self.ip += offset;
                    }
                }
                OpCode::Jump => {
                    let offset = self.read_short() as usize;
                    self.ip += offset;
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

    fn read_short(&mut self) -> u16 {
        self.ip += 2;
        let first = self.chunk.code()[self.ip - 2] as u16;
        let second = self.chunk.code()[self.ip - 1] as u16;
        let res: u16 = first << 8 | second;
        res
    }

    fn read_consnt(&mut self) -> Value {
        let idx = self.read_byte();
        self.chunk.read_constant(idx as usize)
    }

    fn push(&mut self, value: impl Into<Value>) {
        self.stack.push_back(value.into());
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop_back()
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

fn is_falsely(value: Option<&Value>) -> bool {
    match value {
        None => true,
        Some(Value::Nil) => true,
        Some(Value::Bool(bool)) => !bool,
        _ => false,
    }
}

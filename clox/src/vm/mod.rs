use std::collections::{HashMap, VecDeque};
use std::ops::{Div, Mul, Sub};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::value::{Function, NativeFn, NativeFunction};
use crate::{
    chunk::OpCode,
    front::Compiler,
    value::{Object, Value},
};

#[derive(Default)]
struct CallFrame {
    function: Rc<Function>,
    ip: usize,
    slot_idx: usize,
}

impl CallFrame {
    fn new(function: Rc<Function>) -> Self {
        Self {
            function,
            ip: 0,
            slot_idx: 0,
        }
    }

    fn read_byte(&mut self) -> u8 {
        let res = self.function.chunk.read_byte(self.ip);
        self.ip += 1;
        res
    }

    fn read_short(&mut self) -> u16 {
        self.ip += 2;
        let first = self.function.chunk.code()[self.ip - 2] as u16;
        let second = self.function.chunk.code()[self.ip - 1] as u16;
        let res: u16 = first << 8 | second;
        res
    }

    fn read_consnt(&mut self) -> Value {
        let idx = self.read_byte();
        self.function.chunk.read_constant(idx as usize)
    }
}

#[derive(PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
    NativeFunctionError(String),
}

#[derive(Default)]
pub struct Vm {
    stack: VecDeque<Value>,
    frames: VecDeque<CallFrame>,
    cur_frame: CallFrame,
    globals: HashMap<Rc<String>, Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: VecDeque::new(),
            frames: VecDeque::new(),
            globals: HashMap::new(),
            cur_frame: CallFrame::default(),
        }
    }

    pub fn run(&mut self, function: Function) -> InterpretResult {
        let function = Rc::new(function);
        self.push(function.clone());
        match self.call_fun(function, 0) {
            Ok(frame) => self.cur_frame = frame,
            Err(err) => return err,
        }
        loop {
            #[cfg(debug_assertions)]
            {
                print!("    ");
                for val in &self.stack {
                    print!("[ {:?} ]", val);
                }
                if self.stack.is_empty() {
                    print!("[]");
                }
                println!();
                self.cur_frame
                    .function
                    .chunk
                    .disassemble_instruction(self.cur_frame.ip);
            }

            let inst = self.read_byte().into();
            match inst {
                OpCode::Return => match self.pop() {
                    Some(val) => match self.frames.pop_back() {
                        Some(frame) => {
                            let slot_idx = self.cur_frame.slot_idx;
                            self.stack.drain(slot_idx..);
                            self.push(val);
                            self.cur_frame = frame;
                        }
                        None => {
                            self.pop();
                            return InterpretResult::Ok;
                        }
                    },
                    None => {
                        self.runtime_error("method return, stack too short");
                        return InterpretResult::RuntimeError;
                    }
                },
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
                    if let None = self.pop() {
                        self.runtime_error("Stack to too short");
                        return InterpretResult::RuntimeError;
                    }
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
                    if let Some(val) = self.stack.get(self.cur_frame.slot_idx + slot as usize) {
                        self.push(val.clone())
                    } else {
                        self.runtime_error(&format!("getLocal operand error, slot:{}", slot));
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

                    if let Some(local) = self.stack.get_mut(self.cur_frame.slot_idx + slot as usize)
                    {
                        *local = val.clone();
                    } else {
                        self.runtime_error("setLocal target not exits");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_short() as usize;
                    if is_falsely(self.peak(0)) {
                        self.cur_frame.ip += offset;
                    }
                }
                OpCode::JumpIfTrue => {
                    let offset = self.read_short() as usize;
                    if is_truely(self.peak(0)) {
                        self.cur_frame.ip += offset;
                    }
                }
                OpCode::Jump => {
                    let offset = self.read_short() as usize;
                    self.cur_frame.ip += offset;
                }
                OpCode::Loop => {
                    let offset = self.read_short();
                    self.cur_frame.ip -= offset as usize;
                }
                OpCode::Call => {
                    let arg_count = self.read_byte() as usize;
                    match self.call(arg_count) {
                        Ok(()) => {}
                        Err(res) => return res,
                    }
                }
                OpCode::Unknown(a) => {
                    eprintln!("ip:{:?}, byte:{:?}", self.cur_frame.ip, a)
                }
            }
        }
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
        self.frames.clear();
    }

    pub fn init(&mut self) {
        self.reset_stack();
        self.define_native("clock", clock_native)
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

    fn define_native(&mut self, name: &str, function: NativeFn) {
        let name = Rc::new(name.to_string());
        let function: Value = NativeFunction { function }.into();
        self.globals.insert(name, function);
    }

    fn call(&mut self, arg_count: usize) -> Result<(), InterpretResult> {
        if let Some(val) = self.peak(arg_count) {
            match val {
                Value::Obj(Object::Fn(val)) => {
                    let mut frame = self.call_fun(val.clone(), arg_count)?;
                    std::mem::swap(&mut self.cur_frame, &mut frame);
                    self.frames.push_back(frame);
                }
                Value::Obj(Object::NativeFn(val)) => {
                    let arg_idx = self.stack.len() - arg_count;
                    let args = &self.stack.as_slices().0[arg_idx..];
                    let result = match (val.function)(args) {
                        Ok(val) => val,
                        Err(err) => match err {
                            InterpretResult::NativeFunctionError(str) => {
                                self.runtime_error(&str);
                                Value::Nil
                            }
                            InterpretResult::RuntimeError => {
                                self.runtime_error("Invoke Native fn, unknow error");
                                Value::Nil
                            }
                            _ => Value::Nil,
                        },
                    };
                    self.stack.drain(arg_idx..);
                    self.push(result);
                }
                _ => {
                    self.runtime_error("Can only call functions and classes.");
                    return Err(InterpretResult::RuntimeError);
                }
            }
        } else {
            self.runtime_error("call no operand");
            return Err(InterpretResult::RuntimeError);
        }
        Ok(())
    }

    fn call_fun(
        &mut self,
        fun: Rc<Function>,
        arg_count: usize,
    ) -> Result<CallFrame, InterpretResult> {
        if fun.arity != arg_count {
            self.runtime_error(&format!(
                "Expected {} arguments but got {}.",
                fun.arity, arg_count
            ));
            return Err(InterpretResult::RuntimeError);
        }
        let mut frame = CallFrame::new(fun);
        frame.slot_idx = self.stack.len() - arg_count - 1;
        Ok(frame)
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

    fn read_byte(&mut self) -> u8 {
        self.cur_frame.read_byte()
    }

    fn read_short(&mut self) -> u16 {
        self.cur_frame.read_short()
    }

    fn read_consnt(&mut self) -> Value {
        self.cur_frame.read_consnt()
    }

    fn runtime_error(&self, messgae: &str) {
        eprintln!("{}", messgae);

        frame_error_location(&self.cur_frame);
        for frame in self.frames.iter().rev() {
            frame_error_location(frame);
        }
    }
}

fn clock_native(_: &[Value]) -> Result<Value, InterpretResult> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(e) => Ok((e.as_millis() as f64).into()),
        Err(e) => Err(InterpretResult::NativeFunctionError(format!(
            "getTime error, {}",
            e
        ))),
    }
}

fn frame_error_location(frame: &CallFrame) {
    let fun = &frame.function;
    if fun.chunk.code().is_empty() {
        return;
    }
    let offset = frame.ip - 1;
    eprint!("[line {}] in ", fun.chunk.line(offset).unwrap_or(0));
    if fun.name.as_ref() == "" {
        eprintln!("script");
    } else {
        eprintln!("{}()", fun.name);
    }
}

pub fn interpret(source: &str, vm: &mut Vm) -> InterpretResult {
    let compile = Compiler::default();
    if let Some(function) = compile.compile(source) {
        vm.init();
        vm.run(function)
    } else {
        InterpretResult::CompileError
    }
}

fn is_falsely(value: Option<&Value>) -> bool {
    match value {
        None => true,
        Some(Value::Nil) => true,
        Some(Value::Bool(bool)) => !bool,
        Some(_) => false,
    }
}

fn is_truely(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Nil) => false,
        Some(Value::Bool(bool)) => *bool,
        Some(_) => true,
        None => false,
    }
}

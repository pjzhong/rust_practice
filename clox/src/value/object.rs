use crate::{chunk::Chunk, InterpretResult};
use std::cell::{BorrowMutError, RefCell};
use std::fmt::{Debug, Display};
use std::rc::Rc;

use super::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum Object {
    Str(Rc<String>),
    Fn(Rc<Function>),
    NativeFn(Rc<NativeFunction>),
    Closure(Closure),
    UpValue(usize),
}

#[derive(Default)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub upvalue_count: usize,
    pub name: Rc<String>,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && self.name == other.name
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("arity", &self.arity)
            .field("name", &self.name)
            .finish()
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fn {}",
            if self.name.as_ref() != "" {
                &self.name
            } else {
                "<script>"
            }
        )
    }
}

#[derive(Clone, Default, Debug)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalues: Vec<Rc<UpValue>>,
}

impl PartialEq for Closure {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Closure {
    pub fn new(function: Rc<Function>) -> Self {
        Self {
            function,
            upvalues: vec![],
        }
    }
}

pub type NativeFn = fn(args: &[Value]) -> Result<Value, InterpretResult>;

pub struct NativeFunction {
    pub function: NativeFn,
}

impl PartialEq for NativeFunction {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(str) => write!(f, "{}", str),
            Self::Fn(fun) => write!(f, "{}", fun),
            Self::NativeFn(_) => write!(f, "<native fn>"),
            Self::Closure(cl) => write!(f, "<fn {}>", cl.function.name),
            Self::UpValue(_) => write!(f, "upvalue"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct UpValue {
    innner: RefCell<UpValueInner>,
}

impl UpValue {
    pub fn new(location: usize) -> Self {
        Self {
            innner: RefCell::new(UpValueInner { location }),
        }
    }

    pub fn location(&self) -> usize {
        match self.innner.try_borrow() {
            Ok(loc) => loc.location,
            Err(_) => {
                //TODO Is ok to return max???
                usize::MAX
            }
        }
    }

    pub fn set_location(&self, location: usize) -> Result<(), BorrowMutError> {
        match self.innner.try_borrow_mut() {
            Ok(mut inner) => {
                inner.location = location;

                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct UpValueInner {
    location: usize,
}

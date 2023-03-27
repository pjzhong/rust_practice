mod object;

use std::fmt::Debug;
use std::{fmt::Display, rc::Rc};

pub use self::object::{Closure, Function, NativeFn, NativeFunction, Object, UpValue};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Obj(Object),
}

impl Eq for Value {
    fn assert_receiver_is_total_eq(&self) {}
}

impl From<f64> for Value {
    fn from(val: f64) -> Self {
        Value::Number(val)
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Value::Bool(val)
    }
}

impl From<&str> for Value {
    fn from(val: &str) -> Self {
        Value::Obj(Object::Str(Rc::new(val.to_string())))
    }
}

impl From<String> for Value {
    fn from(val: String) -> Self {
        Value::Obj(Object::Str(Rc::new(val)))
    }
}

impl From<Rc<String>> for Value {
    fn from(val: Rc<String>) -> Self {
        Value::Obj(Object::Str(val))
    }
}

impl From<Function> for Value {
    fn from(function: Function) -> Self {
        Value::Obj(Object::Fn(Rc::new(function)))
    }
}

impl From<Closure> for Value {
    fn from(cl: Closure) -> Self {
        Value::Obj(Object::Closure(cl))
    }
}

impl From<Rc<Function>> for Value {
    fn from(function: Rc<Function>) -> Self {
        Value::Obj(Object::Fn(function))
    }
}

impl From<NativeFunction> for Value {
    fn from(function: NativeFunction) -> Self {
        Value::Obj(Object::NativeFn(Rc::new(function)))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(bool) => write!(f, "{}", bool),
            Value::Number(num) => write!(f, "{}", num),
            Value::Obj(a) => std::fmt::Display::fmt(&a, f),
        }
    }
}

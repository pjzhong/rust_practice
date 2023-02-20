use std::{fmt::Display, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Obj(Object),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Object {
    Str(Rc<String>),
}

impl Eq for Value {}

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

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(bool) => write!(f, "{}", bool),
            Value::Number(num) => write!(f, "{}", num),
            obj @ Value::Obj(_) => obj.fmt(f),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(str) => write!(f, "{}", str),
        }
    }
}

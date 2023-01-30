use std::{fmt::Display, rc::Rc};

use super::{
    class::{LoxClass, LoxInstance},
    function::LoxCallable,
};

#[derive(Debug, Clone)]
pub enum LoxValue {
    Number(f64),
    Boolean(bool),
    String(Rc<String>),
    Classs(Rc<LoxClass>),
    Call(LoxCallable),
    Instance(Rc<LoxInstance>),
    Nil,
}

impl Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxValue::Number(a) => write!(f, "{}", a),
            LoxValue::Boolean(a) => write!(f, "{}", a),
            LoxValue::String(a) => write!(f, "{}", a),
            LoxValue::Call(c) => c.fmt(f),
            LoxValue::Nil => write!(f, "nil"),
            LoxValue::Classs(e) => e.fmt(f),
            LoxValue::Instance(i) => i.fmt(f),
        }
    }
}

impl From<String> for LoxValue {
    fn from(a: String) -> Self {
        LoxValue::String(Rc::new(a))
    }
}

impl From<Rc<String>> for LoxValue {
    fn from(a: Rc<String>) -> Self {
        LoxValue::String(a)
    }
}

impl From<bool> for LoxValue {
    fn from(a: bool) -> Self {
        LoxValue::Boolean(a)
    }
}

impl From<&bool> for LoxValue {
    fn from(a: &bool) -> Self {
        LoxValue::Boolean(*a)
    }
}

impl From<f64> for LoxValue {
    fn from(a: f64) -> Self {
        LoxValue::Number(a)
    }
}

impl From<&f64> for LoxValue {
    fn from(a: &f64) -> Self {
        LoxValue::Number(*a)
    }
}

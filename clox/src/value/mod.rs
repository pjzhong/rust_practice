use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
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

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(bool) => write!(f, "{}", bool),
            Value::Number(num) => write!(f, "{}", num),
        }
    }
}

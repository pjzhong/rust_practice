use std::fmt::{Debug, Display};

use crate::{interpreter::LoxValue, Interpreter, LoxErr};

pub trait LoxCallable: Debug {
    fn call(&mut self, interpreter: &mut Interpreter, arguments: Vec<LoxValue>) -> Result<LoxValue, LoxErr>;
}

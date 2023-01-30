use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{token::Token, LoxErr};

use super::LoxValue;

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: Rc<String>,
}

impl From<Rc<String>> for LoxClass {
    fn from(name: Rc<String>) -> Self {
        Self { name }
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "class {}", self.name)
    }
}

#[derive(Debug)]
pub struct LoxInstance {
    klass: Rc<LoxClass>,
    fields: HashMap<Rc<String>, LoxValue>,
}

impl LoxInstance {
    pub fn new(klass: Rc<LoxClass>) -> Self {
        Self {
            klass,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<LoxValue, LoxErr> {
        match self.fields.get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => Err(LoxErr::RunTimeErr(
                Some(name.line),
                format!("Undefined property '{}'.", name.lexeme),
            )),
        }
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.klass.name)
    }
}

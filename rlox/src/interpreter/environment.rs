use crate::{interpreter::LoxValue, token::Token, LoxErr};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Default)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<Arc<String>, LoxValue>,
}

impl Environment {
    pub fn enclosing(enclosing: Option<Environment>) -> Option<Self> {
        Some(Self {
            enclosing: enclosing.map(Box::new),
            values: HashMap::new(),
        })
    }

    pub fn declosing(self) -> Option<Self> {
        self.enclosing.map(|env| *env)
    }

    pub fn define(&mut self, name: Arc<String>, value: LoxValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, token: &Token) -> Result<LoxValue, LoxErr> {
        match self.values.get(&token.lexeme) {
            Some(a) => Ok(a.clone()),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.get(token),
                None => Err(LoxErr::RunTimeErr(
                    Some(token.line),
                    format!("Undefined variable '{}'", &token.lexeme),
                )),
            },
        }
    }

    pub fn assign(&mut self, token: &Token, value: &LoxValue) -> Result<(), LoxErr> {
        match self.values.get_mut(&token.lexeme) {
            Some(val) => {
                *val = value.clone();
                Ok(())
            }
            None => match &mut self.enclosing {
                Some(enclosing) => enclosing.assign(token, value),
                None => Err(LoxErr::RunTimeErr(
                    Some(token.line),
                    format!("Undefined variable '{}'.", token.lexeme),
                )),
            },
        }
    }
}

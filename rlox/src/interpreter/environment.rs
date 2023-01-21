use crate::{interpreter::LoxValue, token::Token, LoxErr};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Default)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<Rc<String>, LoxValue>,
}

impl Environment {
    pub fn enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: Rc<String>, value: LoxValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, token: &Token) -> Result<LoxValue, LoxErr> {
        match self.values.get(&token.lexeme) {
            Some(a) => Ok(a.clone()),
            None => match &self.enclosing {
                Some(enclosing) => match enclosing.try_borrow() {
                    Ok(enclosing) => enclosing.get(token),
                    Err(e) => Err(LoxErr::RunTimeErr(
                        Some(token.line),
                        format!("Undefined variable '{}', err:{}", &token.lexeme, e),
                    )),
                },
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
            None => match &self.enclosing {
                Some(enclosing) => match enclosing.try_borrow_mut() {
                    Ok(mut enclosing) => enclosing.assign(token, value),
                    Err(e) => Err(LoxErr::RunTimeErr(
                        Some(token.line),
                        format!("Undefined variable '{}', err:{}", &token.lexeme, e),
                    )),
                },
                None => Err(LoxErr::RunTimeErr(
                    Some(token.line),
                    format!("Undefined variable '{}'", &token.lexeme),
                )),
            },
        }
    }
}

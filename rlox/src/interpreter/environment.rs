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

    pub fn get_direct(&self, token: &Token) -> Result<LoxValue, LoxErr> {
        match self.values.get(&token.lexeme) {
            Some(a) => Ok(a.clone()),
            None => Err(LoxErr::RunTimeErr(
                Some(token.line),
                format!("Undefined variable '{}'", &token.lexeme),
            )),
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

    pub fn get_at(
        env: Rc<RefCell<Environment>>,
        distance: usize,
        name: &Token,
    ) -> Result<LoxValue, LoxErr> {
        match Environment::ancestor(env, distance).try_borrow() {
            Ok(env) => env.get_direct(name),
            Err(e) => Err(LoxErr::RunTimeErr(
                Some(name.line),
                format!("Undefined variable '{}',e:{}", &name.lexeme, e),
            )),
        }
    }

    pub fn assign_at(
        env: Rc<RefCell<Environment>>,
        distance: usize,
        name: &Token,
        value: &LoxValue,
    ) -> Result<(), LoxErr> {
        match Environment::ancestor(env, distance).try_borrow_mut() {
            Ok(mut env) => match env.values.get_mut(&name.lexeme) {
                Some(val) => {
                    *val = value.clone();
                    Ok(())
                }
                None => Err(LoxErr::RunTimeErr(
                    Some(name.line),
                    format!("Undefined variable '{}'", name.lexeme),
                )),
            },
            Err(e) => Err(LoxErr::RunTimeErr(
                Some(name.line),
                format!("Undefined variable '{}',e:{}", name.lexeme, e),
            )),
        }
    }

    fn ancestor(env: Rc<RefCell<Environment>>, distance: usize) -> Rc<RefCell<Self>> {
        let mut environment = env;
        for _ in 0..distance {
            if let Ok(Some(enclosing)) = environment.try_borrow().map(|e| e.enclosing.clone()) {
                environment = enclosing;
            }
        }

        environment
    }
}

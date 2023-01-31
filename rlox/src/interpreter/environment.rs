use crate::{interpreter::LoxValue, token::Token, LoxErr};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Default)]
pub struct Environment {
    enclosing: Option<Rc<Environment>>,
    inner: RefCell<EnvironmentInner>,
}

#[derive(Debug, Default)]
struct EnvironmentInner {
    values: HashMap<Rc<String>, LoxValue>,
}

impl Environment {
    pub fn enclosing(enclosing: Rc<Environment>) -> Self {
        Self {
            enclosing: Some(enclosing),
            inner: Default::default(),
        }
    }

    pub fn define(&self, name: &Token, value: LoxValue) -> Result<(), LoxErr> {
        match self.inner.try_borrow_mut() {
            Ok(mut inner) => {
                inner.values.insert(name.lexeme.clone(), value);
                Ok(())
            }
            Err(e) => Err(LoxErr::RunTimeErr(
                Some(name.line),
                format!("concurreny exception, define error:{}", e),
            )),
        }
    }

    pub fn str_define(&self, name: Rc<String>, value: LoxValue) -> Result<(), LoxErr> {
        match self.inner.try_borrow_mut() {
            Ok(mut inner) => {
                inner.values.insert(name, value);
                Ok(())
            }
            Err(e) => Err(LoxErr::RunTimeErr(
                None,
                format!("concurreny exception, define error:{}", e),
            )),
        }
    }

    pub fn get(&self, token: &Token) -> Result<LoxValue, LoxErr> {
        match self.inner.try_borrow() {
            Ok(inner) => match inner.values.get(&token.lexeme) {
                Some(a) => Ok(a.clone()),
                None => match &self.enclosing {
                    Some(enclosing) => enclosing.get(token),
                    None => Err(LoxErr::RunTimeErr(
                        Some(token.line),
                        format!("Undefined variable '{}'", &token.lexeme),
                    )),
                },
            },
            Err(e) => Err(LoxErr::RunTimeErr(
                Some(token.line),
                format!("concurreny exception, get error:{}", e),
            )),
        }
    }

    pub fn get_direct(&self, token: &Token) -> Result<LoxValue, LoxErr> {
        match self.inner.try_borrow() {
            Ok(inner) => match inner.values.get(&token.lexeme) {
                Some(a) => Ok(a.clone()),
                None => Err(LoxErr::RunTimeErr(
                    Some(token.line),
                    format!("Undefined variable '{}'", &token.lexeme),
                )),
            },
            Err(e) => Err(LoxErr::RunTimeErr(
                Some(token.line),
                format!("concurreny exception, get error:{}", e),
            )),
        }
        // match self.inner.get(&token.lexeme) {
        //     Some(a) => Ok(a.clone()),
        //     None => Err(LoxErr::RunTimeErr(
        //         Some(token.line),
        //         format!("Undefined variable '{}'", &token.lexeme),
        //     )),
        // }
    }

    pub fn assign(&self, token: &Token, value: &LoxValue) -> Result<(), LoxErr> {
        match self.inner.try_borrow_mut() {
            Ok(mut inner) => match inner.values.get_mut(&token.lexeme) {
                Some(val) => {
                    *val = value.clone();
                    Ok(())
                }
                None => match &self.enclosing {
                    Some(enclosing) => enclosing.assign(token, value),
                    None => Err(LoxErr::RunTimeErr(
                        Some(token.line),
                        format!("Undefined variable '{}'", &token.lexeme),
                    )),
                },
            },
            Err(e) => Err(LoxErr::RunTimeErr(
                Some(token.line),
                format!("concurreny exception, assign error:{}", e),
            )),
        }
    }

    pub fn get_at(
        self: &Rc<Environment>,
        distance: usize,
        token: &Token,
    ) -> Result<LoxValue, LoxErr> {
        let env = self.ancestor(distance);
        let brrow_evn = env.inner.try_borrow();
        match brrow_evn {
            Ok(inner) => match inner.values.get(&token.lexeme) {
                Some(a) => Ok(a.clone()),
                None => Err(LoxErr::RunTimeErr(
                    Some(token.line),
                    format!("Undefined variable '{}'", &token.lexeme),
                )),
            },
            Err(e) => Err(LoxErr::RunTimeErr(
                Some(token.line),
                format!("concurreny exception, get error:{}", e),
            )),
        }
    }

    pub fn get_at_str(
        self: &Rc<Environment>,
        distance: usize,
        token: &Rc<String>,
    ) -> Result<LoxValue, LoxErr> {
        let env = self.ancestor(distance);
        let brrow_evn = env.inner.try_borrow();
        match brrow_evn {
            Ok(inner) => match inner.values.get(token) {
                Some(a) => Ok(a.clone()),
                None => Err(LoxErr::RunTimeErr(
                    None,
                    format!("Undefined variable '{}'", token),
                )),
            },
            Err(e) => Err(LoxErr::RunTimeErr(
                None,
                format!("concurreny exception, get error:{}", e),
            )),
        }
    }

    pub fn assign_at(
        self: &Rc<Environment>,
        distance: usize,
        name: &Token,
        value: &LoxValue,
    ) -> Result<(), LoxErr> {
        let env = self.ancestor(distance);
        let env_inner_mut = env.inner.try_borrow_mut();
        match env_inner_mut {
            Ok(mut inner) => {
                inner.values.insert(name.lexeme.clone(), value.clone());
                Ok(())
            }
            Err(e) => Err(LoxErr::RunTimeErr(
                Some(name.line),
                format!("concurreny exception, define error:{}", e),
            )),
        }
    }

    fn ancestor(self: &Rc<Environment>, distance: usize) -> Rc<Self> {
        let mut environment = self.clone();
        for _ in 0..distance {
            if let Some(enclosing) = environment.enclosing.clone() {
                environment = enclosing;
            }
        }

        environment
    }
}

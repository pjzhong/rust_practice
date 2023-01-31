use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{token::Token, LoxErr};

use super::{function::LoxCallable, LoxValue};

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: Rc<String>,
    methods: HashMap<Rc<String>, LoxValue>,
}

impl From<Rc<String>> for LoxClass {
    fn from(name: Rc<String>) -> Self {
        Self {
            name,
            methods: HashMap::new(),
        }
    }
}

impl LoxClass {
    pub fn new(name: Rc<String>, methods: HashMap<Rc<String>, LoxValue>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: &Rc<String>) -> Option<LoxValue> {
        self.methods.get(name).cloned()
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
    inner: RefCell<LoxInstanceInner>,
}

#[derive(Debug, Default)]
struct LoxInstanceInner {
    fields: HashMap<Rc<String>, LoxValue>,
}

impl LoxInstance {
    pub fn new(klass: Rc<LoxClass>) -> Self {
        Self {
            klass,
            inner: RefCell::new(Default::default()),
        }
    }

    pub fn get(self: &Rc<Self>, name: &Token) -> Result<LoxValue, LoxErr> {
        match self.inner.try_borrow() {
            Ok(val) => val.get(self, name),
            Err(e) => Err(LoxErr::RunTimeErr(
                Some(name.line),
                format!(
                    "Concurrency exception get property '{}'. error:{}",
                    name.lexeme, e
                ),
            )),
        }
    }

    pub fn set(&self, name: &Token, value: LoxValue) -> Result<(), LoxErr> {
        match self.inner.try_borrow_mut() {
            Ok(mut val) => {
                val.set(name, value);
                Ok(())
            }
            Err(e) => Err(LoxErr::RunTimeErr(
                Some(name.line),
                format!(
                    "Concurrency exception set property '{}'. error:{}",
                    name.lexeme, e
                ),
            )),
        }
    }
}

impl LoxInstanceInner {
    pub fn get(&self, inst: &Rc<LoxInstance>, name: &Token) -> Result<LoxValue, LoxErr> {
        match self.fields.get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => match inst.klass.find_method(&name.lexeme) {
                Some(LoxValue::Call(LoxCallable::LoxFun(fun))) => {
                    let new_fun = fun.bind(inst.clone())?;
                    Ok(LoxValue::Call(LoxCallable::LoxFun(new_fun)))
                }
                _ => Err(LoxErr::RunTimeErr(
                    Some(name.line),
                    format!("Undefined property '{}'.", name.lexeme),
                )),
            },
        }
    }

    pub fn set(&mut self, name: &Token, value: LoxValue) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.klass.name)
    }
}

use std::{
    fmt::{Debug, Display},
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::Stmt,
    interpreter::{Environment, LoxValue},
    token::Token,
    Interpreter, LoxErr,
};

use super::class::{LoxClass, LoxInstance};

#[derive(Debug, Clone)]
pub struct LoxFunction {
    pub name: Token,
    pub args: Rc<Vec<Token>>,
    pub body: Rc<Vec<Stmt>>,
    pub closure: Rc<Environment>,
}

impl LoxFunction {
    pub fn bind(&self, instance: Rc<LoxInstance>) -> Result<Self, LoxErr> {
        let envir = Environment::enclosing(self.closure.clone());
        envir.str_define(Rc::new("this".to_string()), LoxValue::Instance(instance))?;

        let mut fun = self.clone();
        fun.closure = Rc::new(envir);

        Ok(fun)
    }
}

pub enum FunctionType {
    None,
    Fn,
    Method,
}

#[derive(Debug, Clone)]
pub enum LoxCallable {
    Clock,
    LoxFun(LoxFunction),
    Class(Rc<LoxClass>),
}

impl LoxCallable {
    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::Clock => 0,
            LoxCallable::LoxFun(fun) => fun.args.len(),
            LoxCallable::Class(_) => 0,
        }
    }

    pub fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxErr> {
        match self {
            LoxCallable::Clock => LoxCallable::clock(),
            LoxCallable::LoxFun(fun) => LoxCallable::lox_call(fun, interpreter, arguments),
            LoxCallable::Class(class) => {
                Ok(LoxValue::Instance(Rc::new(LoxInstance::new(class.clone()))))
            }
        }
    }

    fn clock() -> Result<LoxValue, LoxErr> {
        let ts = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(e) => e.as_millis(),
            Err(e) => return Err(LoxErr::RunTimeErr(None, format!("getTime error, {}", e))),
        };

        Ok(LoxValue::Number(ts as f64))
    }

    fn lox_call(
        fun: &LoxFunction,
        interpreter: &mut Interpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxErr> {
        let environment = Environment::enclosing(fun.closure.clone());
        for (name, value) in fun.args.iter().zip(args) {
            environment.define(name, value)?;
        }

        match interpreter.execute_block(fun.body.as_ref(), environment) {
            Ok(_) => Ok(LoxValue::Nil),
            Err(LoxErr::Return(val)) => Ok(val),
            Err(err) => Err(err),
        }
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxCallable::Clock => write!(f, "native fn clock()"),
            LoxCallable::LoxFun(fun) => write!(f, "fn {}()", fun.name.lexeme),
            LoxCallable::Class(c) => Display::fmt(&c, f),
        }
    }
}

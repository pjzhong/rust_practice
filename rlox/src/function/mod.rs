use std::{
    fmt::{Debug, Display},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::Stmt,
    interpreter::{Environment, LoxValue},
    Interpreter, LoxErr,
};

#[derive(Debug, Clone)]
pub enum LoxCallable {
    Clock,
    LoxFun(Stmt),
}

impl LoxCallable {
    pub fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxErr> {
        match self {
            LoxCallable::Clock => LoxCallable::clock(),
            LoxCallable::LoxFun(stmt) => LoxCallable::lox_call(stmt, interpreter, arguments),
        }
    }

    fn clock() -> Result<LoxValue, LoxErr> {
        let ts = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(e) => e.as_secs(),
            Err(e) => return Err(LoxErr::RunTimeErr(None, format!("getTime error, {}", e))),
        };

        Ok(LoxValue::Number(ts as f64))
    }

    fn lox_call(
        stmt: &Stmt,
        interpreter: &mut Interpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxErr> {
        let (args_tokens, body) = match stmt {
            Stmt::Fun(_, args, body) => (args, body),
            _ => return Err(LoxErr::RunTimeErr(None, "not a function error".to_string())),
        };

        let mut environment = Environment::enclosing(interpreter.global.clone());
        for (name, value) in args_tokens.iter().zip(args) {
            environment.define(name.lexeme.clone(), value);
        }

        interpreter.execute_block(body, environment)?;
        Ok(LoxValue::Nil)
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxCallable::Clock => write!(f, "native fn clock()"),
            LoxCallable::LoxFun(stmt) => match stmt {
                Stmt::Fun(token, ..) => write!(f, "fn {}", token.lexeme),
                _ => write!(f, "not a funtion"),
            },
        }
    }
}

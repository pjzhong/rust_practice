use std::{
    fmt::{Debug, Display},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::Stmt,
    interpreter::{Environment, LoxValue},
    token::Token,
    Interpreter, LoxErr,
};

#[derive(Debug, Clone)]
pub enum LoxCallable {
    Clock,
    LoxFun(Token, Vec<Token>, Vec<Stmt>),
}

impl LoxCallable {
    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::Clock => 0,
            LoxCallable::LoxFun(_, args, _) => args.len(),
        }
    }

    pub fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxErr> {
        match self {
            LoxCallable::Clock => LoxCallable::clock(),
            LoxCallable::LoxFun(_, args, body) => {
                LoxCallable::lox_call(args, body, interpreter, arguments)
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
        arg_tokens: &[Token],
        body: &[Stmt],
        interpreter: &mut Interpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxErr> {
        let mut environment = Environment::enclosing(interpreter.global.clone());
        for (name, value) in arg_tokens.iter().zip(args) {
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
            LoxCallable::LoxFun(token, ..) => write!(f, "fn {}", token.lexeme),
        }
    }
}

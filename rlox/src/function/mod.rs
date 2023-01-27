use std::{
    cell::RefCell,
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

#[derive(Debug, Clone)]
pub enum LoxCallable {
    Clock,
    LoxFun(Token, Vec<Token>, Rc<Vec<Stmt>>, Rc<RefCell<Environment>>),
}

impl LoxCallable {
    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::Clock => 0,
            LoxCallable::LoxFun(_, args, _, _) => args.len(),
        }
    }

    pub fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxErr> {
        match self {
            LoxCallable::Clock => LoxCallable::clock(),
            LoxCallable::LoxFun(_, args, body, closure) => {
                LoxCallable::lox_call(args, body, closure.clone(), interpreter, arguments)
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
        closure: Rc<RefCell<Environment>>,
        interpreter: &mut Interpreter,
        args: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxErr> {
        let mut environment = Environment::enclosing(closure);
        for (name, value) in arg_tokens.iter().zip(args) {
            environment.define(name.lexeme.clone(), value);
        }

        match interpreter.execute_block(body, environment) {
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
            LoxCallable::LoxFun(token, ..) => write!(f, "fn {}()", token.lexeme),
        }
    }
}

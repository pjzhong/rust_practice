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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionType {
    None,
    Fn,
    Method,
    Initializer,
}

#[derive(Debug, Clone)]
pub struct LoxFunction {
    pub name: Token,
    pub args: Rc<Vec<Token>>,
    pub body: Rc<Vec<Stmt>>,
    pub closure: Rc<Environment>,
    pub fun_type: FunctionType,
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

impl From<LoxFunction> for LoxCallable {
    fn from(fun: LoxFunction) -> Self {
        LoxCallable::LoxFun(fun)
    }
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
            LoxCallable::Class(clss) => {
                if let Some(LoxValue::Call(Self::LoxFun(fun))) =
                    clss.find_method(&Rc::new("init".to_string()))
                {
                    fun.args.len()
                } else {
                    0
                }
            }
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
                let inst = Rc::new(LoxInstance::new(class.clone()));
                if let Some(LoxValue::Call(LoxCallable::LoxFun(fun))) =
                    class.find_method(&Rc::new("init".to_string()))
                {
                    let fun = fun.bind(inst.clone())?;
                    LoxCallable::LoxFun(fun).call(interpreter, arguments)?;
                }

                Ok(inst.into())
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
            Ok(_) => {
                if fun.fun_type == FunctionType::Initializer {
                    Ok(fun.closure.get_at_str(0, &Rc::new("this".to_string()))?)
                } else {
                    Ok(LoxValue::Nil)
                }
            }

            Err(LoxErr::Return(val)) => {
                if fun.fun_type == FunctionType::Initializer {
                    Ok(fun.closure.get_at_str(0, &Rc::new("this".to_string()))?)
                } else {
                    Ok(val)
                }
            }
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

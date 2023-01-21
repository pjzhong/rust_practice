mod environment;

use std::{
    cell::RefCell,
    fmt::Display,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    ast::{Expr, Stmt, Visitor},
    function::LoxCallable,
    token::{Literal, Token, TokenType},
    Lox, LoxErr,
};

pub use self::environment::Environment;

#[derive(Debug, Clone)]
pub enum LoxValue {
    Number(f64),
    Boolean(bool),
    String(Arc<String>),
    Call(LoxCallable),
    Nil,
}

impl Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxValue::Number(a) => write!(f, "{}", a),
            LoxValue::Boolean(a) => write!(f, "{}", a),
            LoxValue::String(a) => write!(f, "{}", a),
            LoxValue::Call(_) => write!(f, "callable"),
            LoxValue::Nil => write!(f, "nil"),
        }
    }
}

impl From<String> for LoxValue {
    fn from(a: String) -> Self {
        LoxValue::String(Arc::new(a))
    }
}

impl From<Arc<String>> for LoxValue {
    fn from(a: Arc<String>) -> Self {
        LoxValue::String(a)
    }
}

impl From<bool> for LoxValue {
    fn from(a: bool) -> Self {
        LoxValue::Boolean(a)
    }
}

impl From<&bool> for LoxValue {
    fn from(a: &bool) -> Self {
        LoxValue::Boolean(*a)
    }
}

impl From<f64> for LoxValue {
    fn from(a: f64) -> Self {
        LoxValue::Number(a)
    }
}

impl From<&f64> for LoxValue {
    fn from(a: &f64) -> Self {
        LoxValue::Number(*a)
    }
}

type LoxResult<LoxValue> = Result<LoxValue, LoxErr>;

pub struct Interpreter {
    lox: Rc<Mutex<Lox>>,
    environment: Rc<RefCell<Environment>>,
    pub global: Rc<RefCell<Environment>>,
}

impl Visitor<&Expr, LoxResult<LoxValue>> for Interpreter {
    fn visit(&mut self, expr: &Expr) -> Result<LoxValue, LoxErr> {
        match expr {
            Expr::Literal(a) => self.literal(a),
            Expr::Unary(token, expr) => self.unary(expr, token),
            Expr::Binary(left, oper, right) => self.binary(left, oper, right),
            Expr::Grouping(expr) => self.visit(expr.as_ref()),
            Expr::Variable(token) => {
                if let Ok(env) = &self.environment.try_borrow_mut() {
                    env.get(token)
                } else {
                    self.error(
                        token,
                        format!(
                            "environment is None, Undefined variable '{}'",
                            &token.lexeme
                        ),
                    )
                }
            }
            Expr::Assign(token, value) => {
                let value = self.visit(value.as_ref())?;
                if let Ok(env) = &mut self.environment.try_borrow_mut() {
                    env.assign(token, &value)?;
                } else {
                    return self.error(
                        token,
                        format!(
                            "environment is None, Undefined variable variable '{}'",
                            &token.lexeme
                        ),
                    );
                }
                Ok(value)
            }
            Expr::Logical(left, oper, right) => {
                let left = self.visit(left.as_ref())?;

                if oper.toke_type == TokenType::Or {
                    if self.is_truthy(Some(&left)) {
                        return Ok(left);
                    }
                } else if !self.is_truthy(Some(&left)) {
                    return Ok(left);
                }

                self.visit(right.as_ref())
            }
            Expr::Call(callee, paren, exprs) => {
                let callee = self.visit(callee.as_ref())?;

                let mut args = vec![];
                for expr in exprs {
                    args.push(self.visit(expr)?);
                }

                let mut callee = match callee {
                    LoxValue::Call(callee) => callee,
                    _ => {
                        return Err(LoxErr::RunTimeErr(
                            Some(paren.line),
                            "Can only call functions and classess.".to_string(),
                        ))
                    }
                };

                callee.call(self, args)
            }
        }
    }
}

impl Visitor<&Stmt, Result<(), LoxErr>> for Interpreter {
    fn visit(&mut self, stmt: &Stmt) -> Result<(), LoxErr> {
        match stmt {
            Stmt::Expression(expr) => match self.visit(expr) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            Stmt::Print(expr) => match self.visit(expr) {
                Ok(val) => {
                    println!("{}", val);
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Stmt::Var(token, expr) => {
                let value = if let Some(expr) = expr {
                    self.visit(expr)?
                } else {
                    LoxValue::Nil
                };

                if let Ok(mut env) = self.environment.try_borrow_mut() {
                    env.define(token.lexeme.clone(), value)
                }
                Ok(())
            }
            Stmt::Block(stmts) => {
                if stmts.is_empty() {
                    Ok(())
                } else {
                    self.execute_block(stmts, Environment::enclosing(self.environment.clone()))
                }
            }
            Stmt::If(condition, then_branch, else_branch) => {
                let value = self.visit(condition)?;
                if self.is_truthy(Some(&value)) {
                    self.visit(then_branch.as_ref())?;
                } else if let Some(stmt) = else_branch {
                    self.visit(stmt.as_ref())?;
                }

                Ok(())
            }
            Stmt::While(condition, body) => {
                loop {
                    let value = self.visit(condition)?;
                    if self.is_truthy(Some(&value)) {
                        match self.visit(body.as_ref()) {
                            Err(LoxErr::BreakOutSideLoop) => return Ok(()),
                            a @ Err(_) => return a,
                            Ok(_) => {}
                        };
                    } else {
                        break;
                    }
                }

                Ok(())
            }
            Stmt::Break => Err(LoxErr::BreakOutSideLoop),
            fun @ Stmt::Fun(name, _, _) => match self.environment.try_borrow_mut() {
                Ok(mut evir) => {
                    let callable = LoxValue::Call(LoxCallable::LoxFun(fun.clone()));
                    evir.define(name.lexeme.clone(), callable);
                    Ok(())
                }
                Err(e) => Err(LoxErr::RunTimeErr(
                    None,
                    format!("define fun err。envir err:{}", e),
                )),
            },
        }
    }
}

impl Interpreter {
    pub fn new(lox: Rc<Mutex<Lox>>) -> Self {
        let envir = Rc::new(RefCell::new(Environment::default()));
        Self {
            lox,
            environment: envir.clone(),
            global: envir,
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match self.visit(stmt) {
                Ok(_) => {}
                Err(e) => {
                    if let Ok(mut lox) = self.lox.lock() {
                        lox.lox_error(e);
                    }
                }
            }
        }
    }

    fn literal(&mut self, a: &Literal) -> LoxResult<LoxValue> {
        match a {
            Literal::String(str) => Ok(str.clone().into()),
            Literal::Number(num) => Ok(num.into()),
            Literal::Bool(b) => Ok(b.into()),
            Literal::Nil => Ok(LoxValue::Nil),
        }
    }

    fn unary(&mut self, expr: &Expr, token: &Token) -> LoxResult<LoxValue> {
        let right = self.visit(expr)?;
        match token.toke_type {
            TokenType::Minus => match right {
                LoxValue::Number(right) => Ok((-right).into()),
                _ => self.error(token, String::from("Operand mus be a number.")),
            },
            TokenType::Bang => Ok((!self.is_truthy(Some(&right))).into()),
            _ => self.error(
                token,
                format!(
                    "unsupoort unary operation:{:?},{:?}",
                    token.toke_type, right
                ),
            ),
        }
    }

    fn binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> LoxResult<LoxValue> {
        let left = self.visit(left)?;
        let right = self.visit(right)?;

        match (left, right) {
            (LoxValue::Number(left), LoxValue::Number(right)) => match operator.toke_type {
                TokenType::Minus => Ok((left - right).into()),
                TokenType::Plus => Ok((left + right).into()),
                TokenType::Slash => {
                    if right == 0.0 {
                        self.error(operator, "divied by zero".to_string())
                    } else {
                        Ok((left / right).into())
                    }
                }

                TokenType::Star => Ok((left * right).into()),
                TokenType::Greater => Ok((left > right).into()),
                TokenType::GreaterEqual => Ok((left >= right).into()),
                TokenType::Less => Ok((left < right).into()),
                TokenType::LessEqual => Ok((left <= right).into()),
                TokenType::BangEqual => Ok((left != right).into()),
                TokenType::EqualEqual => Ok((left == right).into()),
                _ => self.error(
                    operator,
                    format!("unsuppoert number operation:{:?}", operator.toke_type),
                ),
            },
            (LoxValue::String(left), LoxValue::String(right)) => match operator.toke_type {
                TokenType::Plus => {
                    let mut str = String::new();
                    str.push_str(&left);
                    str.push_str(&right);
                    Ok(str.into())
                }
                TokenType::BangEqual => Ok((left != right).into()),
                TokenType::EqualEqual => Ok((left == right).into()),
                _ => self.error(
                    operator,
                    format!("unsuppoert string operation:{:?}", operator.toke_type),
                ),
            },
            (left, right) => match operator.toke_type {
                TokenType::BangEqual => Ok((!self.is_equal(&left, &right)).into()),
                TokenType::EqualEqual => Ok((self.is_equal(&left, &right)).into()),
                TokenType::Minus | TokenType::Slash | TokenType::Star => {
                    self.error(operator, "Operands must be numbers.".to_string())
                }

                TokenType::Plus => self.error(
                    operator,
                    "Operands must be two numbers or two strings".to_string(),
                ),
                _ => self.error(
                    operator,
                    format!(
                        "unsupport binary operation:{:?} with:{:?}, {:?}.",
                        operator.toke_type, left, right
                    ),
                ),
            },
        }
    }

    fn is_truthy(&self, val: Option<&LoxValue>) -> bool {
        match val {
            Some(LoxValue::Boolean(val)) => *val,
            Some(LoxValue::Nil) => false,
            Some(_) => true,
            None => false,
        }
    }

    fn is_equal(&self, a: &LoxValue, b: &LoxValue) -> bool {
        match (a, b) {
            (LoxValue::Number(a), LoxValue::Number(b)) => a == b,
            (LoxValue::Boolean(a), LoxValue::Boolean(b)) => a == b,
            (LoxValue::String(a), LoxValue::String(b)) => a == b,
            //TODO How can I sure two Box<dyn Any> is equal
            //Leave it to clox
            (LoxValue::Nil, LoxValue::Nil) => true,
            _ => false,
        }
    }

    fn error(&self, token: &Token, message: String) -> LoxResult<LoxValue> {
        Err(LoxErr::RunTimeErr(Some(token.line), message))
    }

    pub fn execute_block(
        &mut self,
        stmts: &[Stmt],
        environment: Environment,
    ) -> Result<(), LoxErr> {
        let previous = self.environment.clone();

        self.environment = Rc::new(RefCell::new(environment));
        let res = stmts
            .iter()
            .map(|stmt| self.visit(stmt))
            .find(Result::is_err)
            .map_or(Ok(()), Result::from);

        self.environment = previous;
        res
    }
}

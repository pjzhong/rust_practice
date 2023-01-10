use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use crate::{
    ast::Expr,
    token::{Literal, Token, TokenType},
    Lox, LoxErr, Visitor,
};

#[derive(Debug)]
pub enum LoxValue {
    Number(f64),
    Boolean(bool),
    String(String),
    Any(Box<dyn Any>),
    Nil,
}

impl From<String> for LoxValue {
    fn from(a: String) -> Self {
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

impl From<Box<dyn Any>> for LoxValue {
    fn from(a: Box<dyn Any>) -> Self {
        LoxValue::Any(a)
    }
}

type LoxResult<LoxValue> = Result<LoxValue, LoxErr>;

pub struct Interpreter {
    lox: Arc<Mutex<Lox>>,
}

impl Visitor<LoxResult<LoxValue>> for Interpreter {
    fn visit_expr(&self, expr: &Expr) -> Result<LoxValue, LoxErr> {
        match expr {
            Expr::Literal(a) => self.literal(a),
            Expr::Unary(token, expr) => self.unary(expr, token),
            Expr::Binary(left, oper, right) => self.binary(left, oper, right),
            Expr::Grouping(expr) => self.visit_expr(expr),
        }
    }
}

impl Interpreter {
    pub fn new(lox: Arc<Mutex<Lox>>) -> Self {
        Self { lox }
    }

    pub fn interpret(&mut self, expr: &Expr) {
        match self.visit_expr(expr) {
            Ok(val) => println!("{:?}", val),
            Err(e) => {
                if let Ok(mut lox) = self.lox.lock() {
                    lox.runtimne_error(e);
                }
            }
        }
    }

    fn literal(&self, a: &Literal) -> LoxResult<LoxValue> {
        match a {
            Literal::String(str) => Ok(str.clone().into()),
            Literal::Number(num) => Ok(num.into()),
            Literal::Bool(b) => Ok(b.into()),
            Literal::Nil => Ok(LoxValue::Nil),
        }
    }

    fn unary(&self, expr: &Expr, token: &Token) -> LoxResult<LoxValue> {
        let right = self.visit_expr(expr)?;
        match token.toke_type {
            TokenType::Minus => match right {
                LoxValue::Number(right) => Ok((-right).into()),
                _ => self.error(token, String::from("Operand mus be a number.")),
            },
            TokenType::Bang => Ok((!self.is_truthy(&Some(right))).into()),
            _ => self.error(
                token,
                format!("unsupoort unary operation:{:?},{:?}", token.toke_type, right),
            ),
        }
    }

    fn binary(&self, left: &Expr, operator: &Token, right: &Expr) -> LoxResult<LoxValue> {
        let left = self.visit_expr(left)?;
        let right = self.visit_expr(right)?;

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

    fn is_truthy(&self, val: &Option<LoxValue>) -> bool {
        match val {
            Some(LoxValue::Boolean(val)) => *val,
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
}

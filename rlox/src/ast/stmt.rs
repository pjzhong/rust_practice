use crate::token::Token;

use super::expr::Expr;

pub trait Visitor<T, R> {
    fn visit(&mut self, expr: T) -> R;
}

pub enum Stmt {
    Print(Expr),
    Expression(Expr),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
}

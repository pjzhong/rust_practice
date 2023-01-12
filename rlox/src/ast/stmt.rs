use crate::token::Token;

use super::Expr;

pub trait Visitor<T> {
    fn visit_stmt(&self, expr: &Stmt) -> T;
}

pub enum Stmt {
    Print(Expr),
    Expression(Expr),
    Var(Token, Option<Expr>),
}

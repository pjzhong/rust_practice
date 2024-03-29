use std::rc::Rc;

use crate::token::Token;

use super::expr::Expr;

pub trait Visitor<T, R> {
    fn visit(&mut self, expr: T) -> R;
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum Stmt {
    Print(Expr),
    Expression(Expr),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>),
    If(Expr, Rc<Stmt>, Option<Rc<Stmt>>),
    While(Option<Rc<Stmt>>, Expr, Vec<Stmt>),
    Fun(Token, Rc<Vec<Token>>, Rc<Vec<Stmt>>),
    Return(Token, Option<Expr>),
    Break(Token),
    Class(Token, Option<Rc<Expr>>, Rc<Vec<Stmt>>),
}

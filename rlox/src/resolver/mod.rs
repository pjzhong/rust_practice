use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::Mutex,
};

use crate::{
    ast::{Expr, Stmt, Visitor},
    token::Token,
    Interpreter, Lox,
};

pub struct Resolver {
    scopes: VecDeque<HashMap<Rc<String>, bool>>,
    locals: HashMap<Expr, usize>,
    lox: Rc<Mutex<Lox>>,
}

impl Visitor<&Expr, ()> for Resolver {
    fn visit(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable(token) => {
                if let Some(false) = self.scopes.back().and_then(|map| map.get(&token.lexeme)) {
                    self.error(token, "Can't read local variable in its own initializer.");
                    return;
                }

                self.resolve_local(expr, token);
            }
            Expr::Assign(token, value) => {
                self.visit(value.as_ref());
                self.resolve_local(expr, token);
            }
            Expr::Binary(left, _, right) => {
                self.visit(left.as_ref());
                self.visit(right.as_ref());
            }
            Expr::Grouping(expr) => self.visit(expr.as_ref()),
            Expr::Literal(_) => {}
            Expr::Unary(_, expr) => self.visit(expr.as_ref()),
            Expr::Logical(left, _, right) => {
                self.visit(left.as_ref());
                self.visit(right.as_ref());
            }
            Expr::Call(expr, _, exprs) => {
                self.visit(expr.as_ref());
                for expr in exprs {
                    self.visit(expr);
                }
            }
            Expr::Lambda(_, tokens, body) => self.resolve_fun(tokens, body),
        }
    }
}

impl Visitor<&Stmt, ()> for Resolver {
    fn visit(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(stmts) => {
                self.begin_scope();
                for stmt in stmts {
                    self.visit(stmt);
                }
                self.end_scope();
            }
            Stmt::Var(name, initializer) => {
                self.declare(name);
                if let Some(initializer) = initializer {
                    self.visit(initializer)
                }
                self.define(name);
            }
            Stmt::Fun(token, params, body) => {
                self.declare(token);
                self.define(token);

                self.resolve_fun(params, body);
            }
            Stmt::Print(expr) => self.visit(expr),
            Stmt::Expression(expr) => self.visit(expr),
            Stmt::If(cond, then_branch, else_branch) => {
                self.visit(cond);
                self.visit(then_branch.as_ref());
                if let Some(else_branch) = else_branch {
                    self.visit(else_branch.as_ref());
                }
            }
            Stmt::While(init, cond, body) => {
                if let Some(init) = init {
                    self.visit(init.as_ref());
                }
                self.visit(cond);
                self.visit(body.as_slice());
            }
            Stmt::Return(_, expr) => self.visit(expr),
            Stmt::Break => {}
        }
    }
}

impl Visitor<&[Stmt], ()> for Resolver {
    fn visit(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.visit(stmt);
        }
    }
}

impl Resolver {
    pub fn new(lox: Rc<Mutex<Lox>>) -> Self {
        Self {
            scopes: VecDeque::new(),
            locals: HashMap::new(),
            lox,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push_back(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop_back();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(map) = self.scopes.back_mut() {
            map.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(map) = self.scopes.back_mut() {
            map.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        let size = if self.scopes.is_empty() {
            0
        } else {
            self.scopes.len() - 1
        };

        for (idx, map) in self.scopes.iter().enumerate().rev() {
            if map.get(&name.lexeme).is_some() {
                self.locals.insert(expr.clone(), size - idx);
                return;
            }
        }
    }

    fn resolve_fun(&mut self, params: &[Token], body: &[Stmt]) {
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        for stmt in body {
            self.visit(stmt)
        }
        self.end_scope();
    }

    fn error(&mut self, token: &Token, message: &str) {
        if let Ok(mut lox) = self.lox.lock() {
            lox.error(token.line, message);
        }
    }

    pub fn resolve(mut self, stmt: &[Stmt], interpret: &mut Interpreter) {
        self.visit(stmt);
        interpret.resolve(self.locals);
    }
}

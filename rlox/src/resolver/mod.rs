use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use crate::{
    ast::{Expr, Stmt, Visitor},
    interpreter::FunctionType,
    token::Token,
    Interpreter, Lox,
};

pub struct Resolver {
    scopes: VecDeque<HashMap<Rc<String>, bool>>,
    locals: HashMap<Expr, usize>,
    lox: Rc<Lox>,
    loops: usize,
    current_class: ClassType,
    current_function: FunctionType,
}

#[derive(Clone, PartialEq, Eq)]
pub enum ClassType {
    None,
    Class,
    SubClass,
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
                for expr in exprs.as_ref() {
                    self.visit(expr);
                }
            }
            Expr::Lambda(_, tokens, body) => self.resolve_fun(tokens, body, FunctionType::Fn),
            Expr::Get(expr, _) => self.visit(expr.as_ref()),
            Expr::Set(object, _, value) => {
                self.visit(value.as_ref());
                self.visit(object.as_ref());
            }
            Expr::This(token) => {
                if self.current_class == ClassType::None {
                    self.lox
                        .error(token.line, "Can't use use 'this' outside of a class");
                    return;
                }
                self.resolve_local(expr, token);
            }
            Expr::Super(token, _) => {
                if self.current_class == ClassType::None {
                    self.lox
                        .error(token.line, "Can't use use 'super' outside of a class");
                } else if self.current_class != ClassType::SubClass {
                    self.lox.error(
                        token.line,
                        "Can't use use 'super' in a class with no superclass.",
                    );
                }
                self.resolve_local(expr, token);
            }
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

                self.resolve_fun(params, body, FunctionType::Fn);
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
                self.loops += 1;
                if let Some(init) = init {
                    self.visit(init.as_ref());
                }
                self.visit(cond);
                self.visit(body.as_slice());
                self.loops -= 1;
            }
            Stmt::Return(token, expr) => {
                if self.current_function == FunctionType::None {
                    self.lox
                        .error(token.line, "Can't return from top-level code.")
                }

                if let Some(expr) = expr {
                    if self.current_function == FunctionType::Initializer {
                        self.lox
                            .error(token.line, "Can't return a value from an initializer.")
                    }

                    self.visit(expr);
                }
            }
            Stmt::Break(token) => {
                if self.loops == 0 {
                    self.lox.error(token.line, "Can't break outside of loop.")
                }
            }
            Stmt::Class(name, super_cls, methods) => {
                let enclosing_class = self.current_class.clone();
                self.current_class = ClassType::Class;

                self.declare(name);
                self.define(name);

                //This method don't work for cyclic inheritance
                if let Some(super_cls) = super_cls {
                    if let Expr::Variable(token) = super_cls.as_ref() {
                        if token.lexeme == name.lexeme {
                            self.lox
                                .error(token.line, "A class can't inherit from itself.")
                        }
                    }
                    self.current_class = ClassType::SubClass;
                    self.visit(super_cls.as_ref());

                    self.begin_scope();
                    if let Some(scope) = self.scopes.back_mut() {
                        scope.insert(Rc::new("super".to_string()), true);
                    }
                }

                self.begin_scope();
                if let Some(scope) = self.scopes.back_mut() {
                    scope.insert(Rc::new("this".to_string()), true);
                }

                for method in methods.as_ref() {
                    if let Stmt::Fun(token, args, body) = method {
                        self.resolve_fun(
                            args,
                            body,
                            if token.lexeme.as_ref() == "init" {
                                FunctionType::Initializer
                            } else {
                                FunctionType::Method
                            },
                        )
                    }
                }

                if super_cls.is_some() {
                    self.end_scope();
                }

                self.end_scope();
                self.current_class = enclosing_class;
            }
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
    pub fn new(lox: Rc<Lox>) -> Self {
        Self {
            scopes: VecDeque::new(),
            locals: HashMap::new(),
            current_class: ClassType::None,
            current_function: FunctionType::None,
            loops: 0,
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

    fn resolve_fun(&mut self, args: &[Token], body: &[Stmt], fun_type: FunctionType) {
        let enclosing_fun = self.current_function.clone();
        self.current_function = fun_type;
        self.begin_scope();
        for param in args {
            self.declare(param);
            self.define(param);
        }
        for stmt in body {
            self.visit(stmt)
        }
        self.end_scope();
        self.current_function = enclosing_fun;
    }

    fn error(&mut self, token: &Token, message: &str) {
        self.lox.error(token.line, message);
    }

    pub fn resolve(mut self, stmt: &[Stmt], interpret: &mut Interpreter) {
        self.visit(stmt);
        interpret.resolve(self.locals);
    }
}

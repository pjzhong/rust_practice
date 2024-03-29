mod class;
mod environment;
mod function;
mod value;

use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{Expr, Stmt, Visitor},
    token::{Literal, Token, TokenType},
    Lox, LoxErr,
};

pub use self::environment::Environment;
pub use self::function::FunctionType;
pub use self::value::LoxValue;
use self::{
    class::LoxClass,
    function::{LoxCallable, LoxFunction},
};

type LoxResult<LoxValue> = Result<LoxValue, LoxErr>;

pub struct Interpreter {
    lox: Rc<Lox>,
    environment: Rc<Environment>,
    global: Rc<Environment>,
    locals: HashMap<Expr, usize>,
    lambda: usize,
}

impl Visitor<&Expr, LoxResult<LoxValue>> for Interpreter {
    fn visit(&mut self, expr: &Expr) -> Result<LoxValue, LoxErr> {
        match expr {
            Expr::Literal(a) => self.literal(a),
            Expr::Unary(token, expr) => self.unary(expr, token),
            Expr::Binary(left, oper, right) => self.binary(left, oper, right),
            Expr::Grouping(expr) => self.visit(expr.as_ref()),
            Expr::Variable(token) => self.look_up_variable(token, expr),
            Expr::Assign(token, value) => {
                let new_val = self.visit(value.as_ref())?;
                let distance = self.locals.get(expr);

                if let Some(distance) = distance {
                    self.environment.assign_at(*distance, token, &new_val)?;
                } else {
                    self.global.assign(token, &new_val)?;
                }

                Ok(new_val)
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
            Expr::Call(callee, paren, arg_exprs) => {
                let callee = self.visit(callee.as_ref())?;

                let mut args = vec![];
                for expr in arg_exprs.as_ref() {
                    args.push(self.visit(expr)?);
                }

                let mut callee = match callee {
                    LoxValue::Call(callee) => callee,
                    LoxValue::Classs(class) => LoxCallable::Class(class),
                    _ => {
                        return Err(LoxErr::RunTimeErr(
                            Some(paren.line),
                            "Can only call functions and classess.".to_string(),
                        ))
                    }
                };

                if callee.arity() != arg_exprs.len() {
                    return Err(LoxErr::RunTimeErr(
                        Some(paren.line),
                        format!(
                            "Expected {} arguments but got {}.",
                            callee.arity(),
                            args.len()
                        ),
                    ));
                }

                callee.call(self, args)
            }
            Expr::Lambda(token, args, body) => {
                self.lambda += 1;
                let name = Token {
                    toke_type: token.toke_type,
                    lexeme: Rc::new(format!("lambda#{}", self.lambda)),
                    value: Literal::Nil,
                    line: token.line,
                };
                let callee = LoxFunction {
                    name,
                    args: args.clone(),
                    body: body.clone(),
                    closure: self.environment.clone(),
                    fun_type: FunctionType::Fn,
                }
                .into();
                Ok(LoxValue::Call(callee))
            }
            Expr::Get(expr, name) => match self.visit(expr.as_ref())? {
                LoxValue::Instance(inst) => inst.get(name),
                _ => self.error(name, "Only instances have properties.".to_string()),
            },
            Expr::Set(object, name, value) => match self.visit(object.as_ref())? {
                LoxValue::Instance(inst) => {
                    let value = self.visit(value.as_ref())?;
                    inst.set(name, value.clone())?;
                    Ok(value)
                }
                _ => self.error(name, "Only instances have fields".to_string()),
            },
            Expr::This(token) => self.look_up_variable(token, expr),
            Expr::Super(key_word, method) => {
                if let Some(dist) = self.locals.get(expr) {
                    let super_cls = self
                        .environment
                        .get_at_str(*dist, &Rc::new("super".to_string()))?;
                    let super_cls = if let LoxValue::Classs(lox_cls) = super_cls {
                        lox_cls
                    } else {
                        return self.error(key_word, "super shuold be a clss".to_string());
                    };

                    let object = self
                        .environment
                        .get_at_str(dist - 1, &Rc::new("this".to_string()))?;
                    let object = if let LoxValue::Instance(object) = object {
                        object
                    } else {
                        return self.error(key_word, "this shuold be a instance".to_string());
                    };

                    let method = if let Some(LoxValue::Call(LoxCallable::LoxFun(fun))) =
                        super_cls.find_method(&method.lexeme)
                    {
                        fun.bind(object)?
                    } else {
                        return self
                            .error(method, format!("undefined  property,'{}'", method.lexeme));
                    };

                    Ok(LoxValue::Call(method.into()))
                } else {
                    self.error(key_word, "super not resolved".to_string())
                }
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

                self.environment.define(token, value)?;
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
            Stmt::While(init, condition, body) => {
                if let Some(init) = init {
                    self.visit(init.as_ref())?;
                }
                loop {
                    let value = self.visit(condition)?;
                    if self.is_truthy(Some(&value)) {
                        for stmt in body {
                            match self.visit(stmt) {
                                Err(LoxErr::BreakOutSideLoop) => return Ok(()),
                                a @ Err(_) => return a,
                                Ok(_) => {}
                            };
                        }
                    } else {
                        break;
                    }
                }

                Ok(())
            }
            Stmt::Break(_) => Err(LoxErr::BreakOutSideLoop),
            Stmt::Fun(name, args, body) => {
                let callable = LoxValue::Call(
                    LoxFunction {
                        name: name.clone(),
                        args: args.clone(),
                        body: body.clone(),
                        closure: self.environment.clone(),
                        fun_type: FunctionType::Fn,
                    }
                    .into(),
                );
                self.environment.define(name, callable)?;
                Ok(())
            }
            Stmt::Return(_, expr) => {
                let value = if let Some(expr) = expr {
                    self.visit(expr)?
                } else {
                    LoxValue::Nil
                };
                Err(LoxErr::Return(value))
            }
            Stmt::Class(name, super_cls, methods) => {
                let super_cls = if let Some(super_cls) = super_cls {
                    if let LoxValue::Classs(super_cls) = self.visit(super_cls.as_ref())? {
                        self.check_cyclic_inheritance(name, super_cls.clone())?;
                        Some(super_cls)
                    } else {
                        return Err(LoxErr::RunTimeErr(
                            Some(name.line),
                            "Superclass must be a class".to_string(),
                        ));
                    }
                } else {
                    None
                };

                self.environment.define(name, LoxValue::Nil)?;

                if let Some(super_cls) = super_cls.clone() {
                    let environment = Environment::enclosing(self.environment.clone());
                    environment
                        .str_define(Rc::new("super".to_string()), LoxValue::Classs(super_cls))?;
                    self.environment = Rc::new(environment);
                }

                let mut class_methods = HashMap::new();
                for stmt in methods.as_ref() {
                    if let Stmt::Fun(name, args, body) = stmt {
                        let function = LoxFunction {
                            name: name.clone(),
                            args: args.clone(),
                            body: body.clone(),
                            closure: self.environment.clone(),
                            fun_type: if name.lexeme.as_ref() == "init" {
                                FunctionType::Initializer
                            } else {
                                FunctionType::Method
                            },
                        };

                        class_methods.insert(name.lexeme.clone(), LoxValue::Call(function.into()));
                    }
                }

                let class = LoxClass::new(name.lexeme.clone(), super_cls.clone(), class_methods);
                let class = LoxValue::Classs(Rc::new(class));

                if super_cls.is_some() {
                    if let Some(env) = self.environment.enclosing_env() {
                        self.environment = env;
                    }
                }
                self.environment.define(name, class)?;
                Ok(())
            }
        }
    }
}

impl Interpreter {
    pub fn new(lox: Rc<Lox>) -> Self {
        let envir = Environment::default();
        if let Err(e) = envir.define(
            &Token {
                toke_type: TokenType::Fn,
                lexeme: Rc::new("clock".to_string()),
                value: Literal::Nil,
                line: 0,
            },
            LoxValue::Call(LoxCallable::Clock),
        ) {
            lox.lox_error(e)
        }

        let envir = Rc::new(envir);
        Self {
            lox,
            environment: envir.clone(),
            global: envir,
            locals: HashMap::new(),
            lambda: 0,
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match self.visit(stmt) {
                Ok(_) => {}
                Err(e) => {
                    self.lox.lox_error(e);
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
        self.environment = Rc::new(environment);
        // let res = stmts
        //     .iter()
        //     .map(|stmt|
        //         self.visit(stmt)
        //     )
        //     .find(Result::is_err)
        //     .map_or(Ok(()), Result::from);

        let mut res = Ok(());
        for stmt in stmts {
            let temp = self.visit(stmt);
            if temp.is_err() {
                res = temp;
                break;
            }
        }

        self.environment = previous;
        res
    }

    fn look_up_variable(&mut self, name: &Token, expr: &Expr) -> Result<LoxValue, LoxErr> {
        if let Some(distance) = self.locals.get(expr) {
            self.environment.get_at(*distance, name)
        } else {
            self.global.get(name)
        }
    }

    pub fn resolve(&mut self, locals: HashMap<Expr, usize>) {
        for (expr, depth) in locals {
            self.locals.insert(expr, depth);
        }
    }

    fn check_cyclic_inheritance(
        &self,
        name: &Token,
        super_cls: Rc<LoxClass>,
    ) -> Result<(), LoxErr> {
        let mut super_cls = Some(super_cls);
        while let Some(check_cls) = super_cls {
            if name.lexeme == check_cls.name() {
                return Err(LoxErr::RunTimeErr(
                    Some(name.line),
                    "detect cyclic inheritance".to_string(),
                ));
            }
            super_cls = check_cls.super_cls();
        }
        Ok(())
    }
}

mod ast;
mod error;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod token;

use std::cell::RefCell;

pub use ast::AstPrinter;
pub use error::LoxErr;
pub use interpreter::Interpreter;
pub use parser::Parser;
pub use resolver::Resolver;
pub use scanner::Scanner;
use token::TokenType;

#[derive(Default)]
pub struct Lox {
    inner: RefCell<LoxInner>,
}

#[derive(Default)]
struct LoxInner {
    pub has_error: bool,
    pub had_runtime_error: bool,
}

/// TODO Fix me, don't let it panic
impl Lox {
    pub fn lox_error(&self, err: LoxErr) {
        match self.inner.try_borrow_mut() {
            Ok(mut inner) => {
                inner.lox_error(err);
            }
            Err(e) => {
                eprintln!("concurrent exception , ignore err:{}, loxErr:{:?}", e, err);
            }
        }
    }

    pub fn error(&self, line: usize, message: &str) {
        self.report(line, "", message)
    }

    pub fn report(&self, line: usize, location: &str, message: &str) {
        println!("[line {}] Error {}, {}", line, location, message);
        match self.inner.try_borrow_mut() {
            Ok(mut inner) => {
                inner.has_error = true;
            }
            Err(e) => {
                eprintln!("concurrent exception ,set run_time_error ignore, err:{}", e);
            }
        }
    }

    pub fn has_error(&self) -> bool {
        match self.inner.try_borrow() {
            Ok(inner) => inner.has_error,
            Err(_) => false,
        }
    }

    pub fn set_error(&self, err: bool) {
        match self.inner.try_borrow_mut() {
            Ok(mut inner) => {
                inner.has_error = err;
            }
            Err(e) => {
                eprintln!("concurrent exception ,set run_error ignore, err:{}", e);
            }
        }
    }

    pub fn had_runtime_error(&self) -> bool {
        match self.inner.try_borrow() {
            Ok(inner) => inner.had_runtime_error,
            Err(_) => false,
        }
    }
}

impl LoxInner {
    pub fn lox_error(&mut self, err: LoxErr) {
        match err {
            LoxErr::ParseErr(line, ty, lexme, message) => {
                if ty == TokenType::Eof {
                    self.report(line, " at end", &message)
                } else {
                    self.report(line, &format!("at '{}'", lexme), &message)
                }
            }
            LoxErr::RunTimeErr(line, message) => {
                if let Some(line) = line {
                    eprintln!("[line {}] {}", line, message);
                } else {
                    eprintln!("{}", message);
                }
                self.had_runtime_error = true;
            }
            LoxErr::BreakOutSideLoop => {
                eprintln!("Break out side of loop");
                self.had_runtime_error = true;
            }
            LoxErr::Return(_) => {
                eprintln!("Unhandle return");
                self.had_runtime_error = true;
            }
        }
    }

    pub fn report(&mut self, line: usize, location: &str, message: &str) {
        println!("[line {}] Error {}, {}", line, location, message);
        self.has_error = true;
    }
}

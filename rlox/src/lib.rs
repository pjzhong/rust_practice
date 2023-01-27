mod ast;
mod error;
mod function;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod token;

pub use ast::AstPrinter;
pub use error::LoxErr;
pub use interpreter::Interpreter;
pub use parser::Parser;
pub use resolver::Resolver;
pub use scanner::Scanner;
use token::TokenType;

#[derive(Default)]
pub struct Lox {
    pub has_error: bool,
    pub had_runtime_error: bool,
}

impl Lox {
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

    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message)
    }

    pub fn report(&mut self, line: usize, location: &str, message: &str) {
        println!("[line {}] Error {}, {}", line, location, message);
        self.has_error = true;
    }
}

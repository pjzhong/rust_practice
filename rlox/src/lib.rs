mod expr;
mod parser;
mod scanner;
mod token;

pub use scanner::Scanner;
use token::{Token, TokenType};

#[derive(Default)]
pub struct Lox {
    pub has_error: bool,
}

impl Lox {
    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message)
    }

    pub fn error_token(&mut self, token: &Token, message: &str) {
        if token.toke_type == TokenType::Eof {
            self.report(token.line, " at end", message)
        } else {
            self.report(token.line, &format!("at '{};", token.lexeme), message)
        }
    }

    pub fn report(&mut self, line: usize, location: &str, message: &str) {
        println!("[line {:?}] Error {:?}:{:?}", line, location, message);
        self.has_error = true;
    }
}

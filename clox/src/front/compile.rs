use super::{scanner::Scanner, TokenType};

pub struct Compiler;

impl Compiler {
    pub fn compile(&self, source: &str) {
        let mut scanner = Scanner::new(source);
        let mut line = 0;
        loop {
            let token = scanner.scan_token();
            if token.line != line {
                print!("{:04} ", token.line);
                line = token.line;
            } else {
                print!("   | ");
            }

            println!("{:?} '{}'", token.ty, token.str);

            if token.ty == TokenType::Eof {
                break;
            }
        }
    }
}

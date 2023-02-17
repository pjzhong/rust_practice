use crate::{Chunk, OpCode};

use super::{scanner::Scanner, token::Token, TokenType};

#[derive(Default)]
pub struct Compiler {
    scanner: Scanner,
    previous: Option<Token>,
    current: Option<Token>,
    error: bool,
    panic: bool,
    compiling_chunk: Chunk,
}

impl Compiler {
    pub fn compile(mut self, source: &str, chunk: Chunk) -> Option<Chunk> {
        self.init_scanner(source);
        self.compiling_chunk = chunk;
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();
        if self.error {
            None
        } else {
            Some(self.compiling_chunk)
        }
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.compiling_chunk
    }

    fn init_scanner(&mut self, source: &str) {
        self.scanner = Scanner::new(source)
    }

    fn advance(&mut self) {
        self.previous = self.current.take();

        loop {
            self.current = Some(self.scanner.scan_token());
            if let Some(token) = &self.current {
                if token.ty != TokenType::Error {
                    break;
                }

                self.error_at_current(token.str.clone().as_ref())
            }
        }
    }

    fn expression(&self) {}

    fn consume(&mut self, expect: TokenType, msg: &str) {
        if let Some(token) = &self.current {
            if token.ty == expect {
                self.advance();
            } else {
                self.error_at_current(msg);
            }
        } else {
            self.error_at_current(msg);
        }
    }

    fn end_compiler(&mut self) {
        self.emit_return();
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn emit_bytes(&mut self, byte1: impl Into<u8>, byte2: impl Into<u8>) {
        self.emit_byte(byte1.into());
        self.emit_byte(byte2.into());
    }

    fn emit_byte(&mut self, byte: impl Into<u8>) {
        let line = self.previous.as_ref().map_or(0, |t| t.line);
        self.current_chunk().write(byte, line);
    }

    fn error(&mut self, message: &str) {
        if self.panic {
            return;
        }
        self.panic = true;

        error_at(&self.previous, message);
        self.error = true;
    }

    fn error_at_current(&mut self, message: &str) {
        if self.panic {
            return;
        }
        self.panic = true;

        error_at(&self.current, message);
        self.error = true;
    }
}

fn error_at(token: &Option<Token>, message: &str) {
    if let Some(token) = token {
        eprint!("[line {}] Error", token.line);

        match token.ty {
            TokenType::Eof => eprint!(" at end"),
            TokenType::Error => {}
            _ => eprint!(" at {}", token.str),
        }
        eprintln!(": {}", message);
    }
}

use crate::{Chunk, OpCode, Value};

use super::{scanner::Scanner, token::Token, TokenType};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Class,      // . ()
    Primary,
}

impl Precedence {
    fn heigher(&self) -> Self {
        match self {
            Precedence::None => Self::Assignment,
            Precedence::Assignment => Self::Or,
            Precedence::Or => Self::And,
            Precedence::And => Self::Equality,
            Precedence::Equality => Self::Comparison,
            Precedence::Comparison => Self::Term,
            Precedence::Term => Self::Factor,
            Precedence::Factor => Self::Unary,
            Precedence::Unary => Self::Class,
            Precedence::Class => Self::Primary,
            Precedence::Primary => Self::Primary,
        }
    }
}

type ParseFn = fn(&mut Compiler);

struct ParseRule {
    prefix: ParseFn,
    infix: ParseFn,
    precedence: Precedence,
}

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

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

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

        #[cfg(debug_assertions)]
        {
            if !self.error {
                self.current_chunk().disassemble_chunk("code");
            }
        }
    }

    fn none(&mut self) {}

    fn binary(&mut self) {
        if let Some(ty) = self.previous.as_ref().map(|t| t.ty) {
            let rule = get_rule(ty);
            self.parse_precedence(rule.precedence.heigher());

            match ty {
                TokenType::Plus => self.emit_byte(OpCode::Add),
                TokenType::Minus => self.emit_byte(OpCode::Subtract),
                TokenType::Star => self.emit_byte(OpCode::Multiply),
                TokenType::Slash => self.emit_byte(OpCode::Divide),
                _ => {}
            }
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self) {
        if let Some(token) = self.previous.as_ref() {
            match token.str.parse::<f64>() {
                Ok(value) => self.emit_constant(value),
                Err(_) => self.error("Illegal number"),
            }
        }
    }

    fn unary(&mut self) {
        self.parse_precedence(Precedence::Unary);
        match self.previous {
            Some(Token {
                ty: TokenType::Minus,
                ..
            }) => self.emit_byte(OpCode::Negate),
            _ => {}
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        if let Some(rule) = self.previous.as_ref().map(|t| get_rule(t.ty)) {
            let prefix = rule.prefix;
            prefix(self);

            while self
                .current
                .as_ref()
                .map_or(false, |t| precedence <= get_rule(t.ty).precedence)
            {
                self.advance();
                if let Some(rule) = self.previous.as_ref().map(|t| get_rule(t.ty)) {
                    let infix = rule.infix;
                    infix(self);
                }
            }
        } else {
            self.error("Expect expression.")
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let const_idx = self.make_constant(value);
        self.emit_bytes(OpCode::Constant, const_idx);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk().add_constant(value);
        if constant > u8::MAX as usize {
            self.error("Too many constant in one chunk.");
            return 0;
        }

        constant as u8
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

fn get_rule(ty: TokenType) -> ParseRule {
    const LEFT_PARAM: ParseRule = ParseRule {
        prefix: Compiler::grouping,
        infix: Compiler::none,
        precedence: Precedence::None,
    };
    const MINUS: ParseRule = ParseRule {
        prefix: Compiler::unary,
        infix: Compiler::binary,
        precedence: Precedence::Term,
    };
    const PLUS: ParseRule = ParseRule {
        prefix: Compiler::none,
        infix: Compiler::binary,
        precedence: Precedence::Term,
    };
    const SLASH: ParseRule = ParseRule {
        prefix: Compiler::none,
        infix: Compiler::binary,
        precedence: Precedence::Factor,
    };
    const STAR: ParseRule = ParseRule {
        prefix: Compiler::none,
        infix: Compiler::binary,
        precedence: Precedence::Factor,
    };
    const NUMBER: ParseRule = ParseRule {
        prefix: Compiler::number,
        infix: Compiler::none,
        precedence: Precedence::None,
    };
    const NONE: ParseRule = ParseRule {
        prefix: Compiler::none,
        infix: Compiler::none,
        precedence: Precedence::None,
    };
    match ty {
        TokenType::LeftParen => LEFT_PARAM,
        TokenType::Minus => MINUS,
        TokenType::Plus => PLUS,
        TokenType::Slash => SLASH,
        TokenType::Star => STAR,
        TokenType::Number => NUMBER,
        _ => NONE,
    }
}

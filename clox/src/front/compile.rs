use crate::{Chunk, OpCode, Value};

use super::{scanner::Scanner, token::Token, TokenType};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        while self.match_advance(TokenType::Eof).is_none() {
            self.declaration()
        }
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

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_advance(TokenType::Equal).is_some() {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil)
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        if let Some(global) = global {
            self.define_variable(global);
        }
    }

    fn declaration(&mut self) {
        match self.match_advance(TokenType::Var) {
            Some(Token {
                ty: TokenType::Var, ..
            }) => self.var_declaration(),
            _ => self.statement(),
        }
        self.statement();

        if self.panic {
            self.synchronize()
        }
    }

    fn synchronize(&mut self) {
        self.panic = false;

        while self
            .current
            .as_ref()
            .map_or(false, |t| t.ty != TokenType::Eof)
        {
            if self
                .previous
                .as_ref()
                .map_or(false, |t| t.ty == TokenType::Semicolon)
            {
                return;
            }

            match self.current.as_ref().map(|t| t.ty) {
                Some(
                    TokenType::Class
                    | TokenType::Fn
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return,
                ) => return,
                _ => {}
            }
        }
    }

    fn statement(&mut self) {
        match self.match_advance(TokenType::Print) {
            Some(Token {
                ty: TokenType::Print,
                ..
            }) => self.print_statement(),
            _ => {}
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value");
        self.emit_byte(OpCode::Print)
    }

    fn consume(&mut self, expect: TokenType, msg: &str) -> Option<&Token> {
        if let Some(token) = &self.current {
            if token.ty == expect {
                self.advance();
                self.previous.as_ref()
            } else {
                self.error_at_current(msg);
                None
            }
        } else {
            self.error_at_current(msg);
            None
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
                TokenType::EqualEqual => self.emit_byte(OpCode::Equal),
                TokenType::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Bang),
                TokenType::Greater => self.emit_byte(OpCode::Greater),
                TokenType::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Bang),
                TokenType::Less => self.emit_byte(OpCode::Less),
                TokenType::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Bang),
                _ => {}
            }
        }
    }

    fn literal(&mut self) {
        match self.previous.as_ref().map(|t| t.ty) {
            Some(TokenType::False) => self.emit_byte(OpCode::False),
            Some(TokenType::Ture) => self.emit_byte(OpCode::True),
            Some(TokenType::Nil) => self.emit_byte(OpCode::Nil),
            _ => {}
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

    fn string(&mut self) {
        if let Some(str) = self.previous.as_ref().map(|t| t.str.clone()) {
            self.emit_constant(&str[1..(str.len() - 1)]);
        }
    }

    fn unary(&mut self) {
        let ty = self.previous.as_ref().map(|t| t.ty);
        self.parse_precedence(Precedence::Unary);
        match ty {
            Some(TokenType::Minus) => self.emit_byte(OpCode::Negate),
            Some(TokenType::Bang) => self.emit_byte(OpCode::Bang),
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

    fn parse_variable(&mut self, message: &str) -> Option<u8> {
        self.consume(TokenType::Identifier, message);
        if let Some(token) = self.previous.as_ref() {
            Some(self.make_constant(token.str.clone()))
        } else {
            None
        }
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(OpCode::DefineGlobal, global);
    }

    fn emit_constant(&mut self, value: impl Into<Value>) {
        let const_idx = self.make_constant(value.into());
        self.emit_bytes(OpCode::Constant, const_idx);
    }

    fn make_constant(&mut self, value: impl Into<Value>) -> u8 {
        let constant = self.current_chunk().add_constant(value.into());
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

    fn check(&self, ty: TokenType) -> bool {
        self.current.as_ref().map_or(false, |t| t.ty == ty)
    }

    fn match_advance(&mut self, ty: TokenType) -> Option<&Token> {
        if self.check(ty) {
            self.advance();
            self.previous.as_ref()
        } else {
            None
        }
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
    const LEFT_PAREN: ParseRule = ParseRule {
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
    const BOOL: ParseRule = ParseRule {
        prefix: Compiler::literal,
        infix: Compiler::none,
        precedence: Precedence::None,
    };
    const NIL: ParseRule = ParseRule {
        prefix: Compiler::literal,
        infix: Compiler::none,
        precedence: Precedence::None,
    };
    const BANG: ParseRule = ParseRule {
        prefix: Compiler::unary,
        infix: Compiler::none,
        precedence: Precedence::None,
    };
    const BANG_EQUAL: ParseRule = ParseRule {
        prefix: Compiler::none,
        infix: Compiler::binary,
        precedence: Precedence::Equality,
    };
    const EQUAL_EQUAL: ParseRule = ParseRule {
        prefix: Compiler::none,
        infix: Compiler::binary,
        precedence: Precedence::Equality,
    };
    const GREATER: ParseRule = ParseRule {
        prefix: Compiler::none,
        infix: Compiler::binary,
        precedence: Precedence::Comparison,
    };
    const GREATER_EQUAL: ParseRule = ParseRule {
        prefix: Compiler::none,
        infix: Compiler::binary,
        precedence: Precedence::Comparison,
    };
    const LESS: ParseRule = ParseRule {
        prefix: Compiler::none,
        infix: Compiler::binary,
        precedence: Precedence::Comparison,
    };
    const LESS_EQUAL: ParseRule = ParseRule {
        prefix: Compiler::none,
        infix: Compiler::binary,
        precedence: Precedence::Comparison,
    };
    const STRING: ParseRule = ParseRule {
        prefix: Compiler::string,
        infix: Compiler::none,
        precedence: Precedence::None,
    };
    const NONE: ParseRule = ParseRule {
        prefix: Compiler::none,
        infix: Compiler::none,
        precedence: Precedence::None,
    };
    match ty {
        TokenType::LeftParen => LEFT_PAREN,
        TokenType::Minus => MINUS,
        TokenType::Plus => PLUS,
        TokenType::Slash => SLASH,
        TokenType::Star => STAR,
        TokenType::Number => NUMBER,
        TokenType::Ture => BOOL,
        TokenType::False => BOOL,
        TokenType::Nil => NIL,
        TokenType::Bang => BANG,
        TokenType::BangEqual => BANG_EQUAL,
        TokenType::EqualEqual => EQUAL_EQUAL,
        TokenType::Greater => GREATER,
        TokenType::GreaterEqual => GREATER_EQUAL,
        TokenType::Less => LESS,
        TokenType::LessEqual => LESS_EQUAL,
        TokenType::String => STRING,
        _ => NONE,
    }
}

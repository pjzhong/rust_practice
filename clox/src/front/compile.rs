use crate::front::precedence::Precedence;
use crate::{value::Function, Chunk, OpCode, Value};
use std::rc::Rc;

use super::precedence::get_rule;
use super::{scanner::Scanner, token::Token, TokenType};

pub type ParseFn = fn(&mut Compiler, bool);

struct Local {
    name: Token,
    depth: i32,
}

enum FunctionType {
    Fn,
    Script,
}

impl Default for FunctionType {
    fn default() -> Self {
        Self::Script
    }
}

#[derive(Default)]
struct Parser {
    error: bool,
    panic: bool,
    previous: Option<Token>,
    current: Option<Token>,
}

impl Parser {
    fn error_at_current(&mut self, message: &str) {
        if self.panic {
            return;
        }
        self.panic = true;
        error_at(&self.current, message);
        self.error = true;
    }
}

#[derive(Default)]
pub struct Compiler {
    parser: Option<Parser>,
    scanner: Option<Scanner>,
    locals: Vec<Local>,
    scope_depth: i32,
    function: Function,
    fn_type: FunctionType,
}

impl Compiler {
    pub fn compile(mut self, source: &str) -> Option<Function> {
        let scanner = self.init_scanner(source);
        self.init_compiler(Some(Parser::default()), Some(scanner), FunctionType::Script);
        self.advance();
        while self.match_advance(TokenType::Eof).is_none()  && self.current().is_some() {
            self.declaration()
        }
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler()
    }

    fn init_compiler(
        &mut self,
        parser: Option<Parser>,
        scanner: Option<Scanner>,
        fn_type: FunctionType,
    ) {
        self.scanner = scanner;
        self.parser = parser;
        self.fn_type = fn_type;
        self.scope_depth = 0;
        // self.locals.push(Local {
        //     depth: 0,
        //     name: Token {
        //         ty: TokenType::None,
        //         str: Rc::new(String::new()),
        //         line: 0,
        //     },
        // })
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    fn init_scanner(&mut self, source: &str) -> Scanner {
        Scanner::new(source)
    }

    fn advance(&mut self) -> Option<&Token> {
        let parser = self.parser.as_mut()?;

        parser.previous = parser.current.take();

        if let Some(scanner) = self.scanner.as_mut() {
            loop {
                let current = scanner.scan_token();
                let token_ty = current.ty;
                if token_ty == TokenType::Error {
                    parser.error_at_current(&current.str);
                }

                parser.current = Some(current);
                if token_ty != TokenType::Error {
                    break;
                }
            }
        } else {
            parser.current = Some(Token {
                ty: TokenType::Eof,
                str: Rc::new(String::new()),
                line: 0,
            });
        }

        parser.previous.as_ref()
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop);
    }

    fn if_statement(&mut self) {
        self.expression();

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);

        if self.match_advance(TokenType::LeftBrace).is_some() {
            self.block();
            self.end_scope();
        } else {
            self.error_at_current("If expect a block");
        }

        let else_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(then_jump);

        //if condition is false, it would execute en implicit pop
        self.emit_byte(OpCode::Pop);
        if self.match_advance(TokenType::Else).is_some() {
            match self.match_advances(&[TokenType::LeftBrace, TokenType::If]) {
                Some(Token {
                    ty: TokenType::LeftBrace,
                    ..
                }) => {
                    self.begin_scope();
                    self.block();
                    self.end_scope();
                }
                Some(Token {
                    ty: TokenType::If, ..
                }) => self.if_statement(),
                _ => self.error_at_current("else expect a block"),
            }
        }
        self.patch_jump(else_jump);
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().code().len();
        self.expression();

        let expt_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);

        if self.match_advance(TokenType::LeftBrace).is_some() {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.error_at_current("while expect a block");
        }

        self.emit_loop(loop_start);
        self.patch_jump(expt_jump);
        self.emit_byte(OpCode::Pop);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        // initializer
        match self.match_advances(&[TokenType::Semicolon, TokenType::Var]) {
            Some(Token {
                ty: TokenType::Semicolon,
                ..
            }) => {
                // no initliazer
            }
            Some(Token {
                ty: TokenType::Var, ..
            }) => self.var_declaration(),

            _ => self.expression_statement(),
        }

        let mut loop_start = self.current_chunk().code().len();

        // condition
        let exit_jump = if self.match_advance(TokenType::Semicolon).is_none() {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition");

            let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
            self.emit_byte(OpCode::Pop);
            Some(exit_jump)
        } else {
            None
        };

        // incrementer
        if !self.check(&TokenType::LeftBrace) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_chunk().code().len();
            self.expression();
            self.emit_byte(OpCode::Pop);

            // jump to the condition clause
            self.emit_loop(loop_start);

            // change the last last of loop body, make it jump to the incrementer
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        //body
        if self.match_advance(TokenType::LeftBrace).is_some() {
            self.block();
        } else {
            self.error_at_current("for expect a block");
        }

        // jump to start of body, condition(if exists) clause incrementer(if exists), modify me if the code change
        self.emit_loop(loop_start);
        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump);
            self.emit_byte(OpCode::Pop);
        }
        self.end_scope();
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

    fn fn_declaration(&mut self) {
        let global = self.parse_variable("Expect function name.");
        self.function(FunctionType::Fn);
        if let Some(global) = global {
            self.define_variable(global);
        }
    }

    fn declaration(&mut self) {
        match self.match_advances(&[TokenType::Var, TokenType::Fn]) {
            Some(Token {
                ty: TokenType::Var, ..
            }) => self.var_declaration(),
            Some(Token {
                ty: TokenType::Fn, ..
            }) => self.fn_declaration(),
            _ => self.statement(),
        }

        if self.parser.as_ref().map_or(false, |f| f.panic) {
            self.synchronize()
        }
    }

    fn synchronize(&mut self) {
        if let Some(parser) = self.parser.as_mut() {
            parser.panic = false;
        }

        while self
            .current()
            .as_ref()
            .map_or(false, |t| t.ty != TokenType::Eof)
        {
            if self
                .previous()
                .as_ref()
                .map_or(false, |t| t.ty == TokenType::Semicolon)
            {
                return;
            }

            match self.current().as_ref().map(|t| t.ty) {
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
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn statement(&mut self) {
        match self.match_advances(&[
            TokenType::Print,
            TokenType::LeftBrace,
            TokenType::If,
            TokenType::While,
            TokenType::For,
        ]) {
            Some(Token {
                ty: TokenType::Print,
                ..
            }) => self.print_statement(),
            Some(Token {
                ty: TokenType::LeftBrace,
                ..
            }) => {
                self.begin_scope();
                self.block();
                self.end_scope();
            }
            Some(Token {
                ty: TokenType::If, ..
            }) => self.if_statement(),
            Some(Token {
                ty: TokenType::While,
                ..
            }) => self.while_statement(),
            Some(Token {
                ty: TokenType::For, ..
            }) => self.for_statement(),
            _ => {
                self.expression_statement();
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        let count = self
            .locals
            .iter()
            .rev()
            .filter(|local| local.depth > self.scope_depth)
            .count();
        for _ in 0..count {
            self.locals.pop();
            self.emit_byte(OpCode::Pop);
        }
    }

    fn block(&mut self) {
        while !self.check(&TokenType::RightBrace) && !self.check(&TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block");
    }

    fn function(&mut self, fn_type: FunctionType) {
        let mut compiler = Compiler::default();
        compiler.init_compiler(self.parser.take(), self.scanner.take(), fn_type);
        compiler.begin_scope();

        compiler.consume(TokenType::LeftParen, "Expect '(' after function name.");
        compiler.consume(TokenType::RightParen, "Expect ')' after parameters.");
        compiler.consume(TokenType::LeftBrace, "Expect '{' before function body.");

        // collect info
        self.parser = compiler.parser.take();
        self.scanner = compiler.scanner.take();
        // return to current compiler
        if let Some(function) = compiler.end_compiler() {
            let idx = self.make_constant(function);
            self.emit_bytes(OpCode::Constant, idx);
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value");
        self.emit_byte(OpCode::Print)
    }

    fn consume(&mut self, expect: TokenType, msg: &str) -> Option<&Token> {
        if let Some(token) = &self.current() {
            if token.ty == expect {
                self.advance()
            } else {
                self.error_at_current(msg);
                None
            }
        } else {
            self.error_at_current(msg);
            None
        }
    }

    fn end_compiler(mut self) -> Option<Function> {
        self.emit_return();

        #[cfg(debug_assertions)]
        {
            if !self.is_error() {
                let name = if self.function.name.as_ref() != "" {
                    self.function.name.clone()
                } else {
                    Rc::new(String::from("<script>"))
                };
                self.current_chunk().disassemble_chunk(name.as_ref());
                println!()
            }
        }

        if self.is_error() {
            None
        } else {
            Some(self.function)
        }
    }

    pub fn binary(&mut self, _: bool) {
        if let Some(ty) = self.previous().map(|t| t.ty) {
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

    pub fn literal(&mut self, _: bool) {
        match self.previous().map(|t| t.ty) {
            Some(TokenType::False) => self.emit_byte(OpCode::False),
            Some(TokenType::Ture) => self.emit_byte(OpCode::True),
            Some(TokenType::Nil) => self.emit_byte(OpCode::Nil),
            _ => {}
        }
    }

    pub fn grouping(&mut self, _: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    pub fn number(&mut self, _: bool) {
        if let Some(token) = self.previous() {
            match token.str.parse::<f64>() {
                Ok(value) => self.emit_constant(value),
                Err(_) => self.error("Illegal number"),
            }
        }
    }

    pub fn string(&mut self, _: bool) {
        if let Some(str) = self.previous().map(|t| t.str.clone()) {
            self.emit_constant(&str[1..(str.len() - 1)]);
        }
    }

    pub fn varaible(&mut self, can_assign: bool) {
        if let Some(token) = self.previous() {
            self.named_varaible(token.str.clone(), can_assign);
        }
    }

    fn named_varaible(&mut self, name: Rc<String>, can_assign: bool) {
        let arg = self.resolve_local(name.as_ref());

        let (arg, get_op, set_op) = if arg != -1 {
            (arg as u8, OpCode::GetLocal, OpCode::SetLocal)
        } else {
            (
                self.identifier_constant(name),
                OpCode::GetGlobal,
                OpCode::SetGlobal,
            )
        };

        if can_assign && self.match_advance(TokenType::Equal).is_some() {
            self.expression();
            self.emit_bytes(set_op, arg);
        } else {
            self.emit_bytes(get_op, arg)
        }
    }

    pub fn unary(&mut self, _: bool) {
        let ty = self.previous().map(|t| t.ty);
        self.parse_precedence(Precedence::Unary);
        match ty {
            Some(TokenType::Minus) => self.emit_byte(OpCode::Negate),
            Some(TokenType::Bang) => self.emit_byte(OpCode::Bang),
            _ => {}
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        if let Some(rule) = self.previous().map(|t| get_rule(t.ty)) {
            let prefix = rule.prefix;
            let can_assign = precedence <= Precedence::Assignment;
            prefix(self, can_assign);

            while self
                .current()
                .as_ref()
                .map_or(false, |t| precedence <= get_rule(t.ty).precedence)
            {
                self.advance();
                if let Some(rule) = self.previous().map(|t| get_rule(t.ty)) {
                    let infix = rule.infix;
                    infix(self, can_assign);
                }

                if can_assign && self.match_advance(TokenType::Equal).is_some() {
                    self.error("Invalid assignment target.");
                }
            }
        } else {
            self.error("Expect expression.")
        }
    }

    fn parse_variable(&mut self, message: &str) -> Option<u8> {
        self.consume(TokenType::Identifier, message);

        self.declare_varaible();
        if self.scope_depth > 0 {
            return None;
        }

        self.previous()
            .as_ref()
            .map(|t| t.str.clone())
            .map(|name| self.identifier_constant(name))
    }

    fn identifier_constant(&mut self, str: Rc<String>) -> u8 {
        self.make_constant(str)
    }

    fn resolve_local(&self, name: &String) -> i32 {
        for (idx, local) in self.locals.iter().enumerate().rev() {
            if local.name.str.as_str() == name {
                return idx as i32;
            }
        }

        -1
    }

    fn declare_varaible(&mut self) {
        if self.scope_depth == 0 {
            return;
        }

        if let Some(token) = self.previous() {
            self.add_local(token.clone())
        } else {
            self.error("Uknow know varaible");
        }
    }

    fn add_local(&mut self, name: Token) {
        if self.locals.len() >= u8::MAX.into() {
            self.error("Too many local varaibles in fucntion.");
            return;
        }

        let local = Local {
            name,
            depth: self.scope_depth,
        };
        self.locals.push(local);
    }

    fn define_variable(&mut self, global: u8) {
        if self.scope_depth > 0 {
            return;
        }

        self.emit_bytes(OpCode::DefineGlobal, global);
    }

    pub fn and(&mut self, _: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    pub fn or(&mut self, _: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfTrue);
        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::Or);

        self.patch_jump(end_jump);
    }

    fn emit_constant(&mut self, value: impl Into<Value>) {
        let const_idx = self.make_constant(value.into());
        self.emit_bytes(OpCode::Constant, const_idx);
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_chunk().code().len() - offset - 2;

        if jump >= u16::MAX.into() {
            self.error("Too much code jump over.")
        }

        let code = self.current_chunk().code();
        code[offset] = ((jump >> 8) & 0xff) as u8;
        code[offset + 1] = (jump & 0xff) as u8
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

    fn emit_jump(&mut self, instruction: impl Into<u8>) -> usize {
        self.emit_byte(instruction.into());
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.current_chunk().code().len() - 2
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OpCode::Loop);

        let offset = (self.current_chunk().code().len() - loop_start + 2) as u16;

        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn emit_byte(&mut self, byte: impl Into<u8>) {
        let line = self.previous().map_or(0, |t| t.line);
        self.current_chunk().write(byte, line);
    }

    fn check(&self, ty: &TokenType) -> bool {
        self.current().as_ref().map_or(false, |t| t.ty == *ty)
    }

    fn current(&self) -> Option<&Token> {
        self.parser.as_ref().and_then(|f| f.current.as_ref())
    }

    fn previous(&self) -> Option<&Token> {
        self.parser.as_ref().and_then(|f| f.previous.as_ref())
    }

    fn is_error(&self) -> bool {
        self.parser.as_ref().map_or(false, |f| f.error)
    }

    fn match_advance(&mut self, ty: TokenType) -> Option<&Token> {
        if self.check(&ty) {
            self.advance()
        } else {
            None
        }
    }

    fn match_advances(&mut self, tys: &[TokenType]) -> Option<&Token> {
        for ty in tys {
            if self.check(ty) {
                return self.advance();
            }
        }
        None
    }

    fn error(&mut self, message: &str) {
        if let Some(parser) = self.parser.as_mut() {
            if parser.panic {
                return;
            }
            parser.panic = true;

            error_at(&parser.previous, message);
            parser.error = true;
        }
    }

    fn error_at_current(&mut self, message: &str) {
        if let Some(parser) = self.parser.as_mut() {
            if parser.panic {
                return;
            }
            parser.panic = true;
            error_at(&parser.current, message);
            parser.error = true;
        }
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

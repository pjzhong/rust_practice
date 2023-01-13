use std::sync::{Arc, Mutex};

use crate::{
    token::{Literal, Token, TokenType},
    Lox,
};

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    lox: Arc<Mutex<Lox>>,
}

impl Scanner {
    pub fn new(source: &str, lox: Arc<Mutex<Lox>>) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            lox,
        }
    }

    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.do_scan_tokens();
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            Default::default(),
            Literal::Nil,
            self.line,
        ));
        self.tokens
    }

    fn do_scan_tokens(&mut self) {
        match self.advance() {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightParen),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let toekn = if self.advance_if_match('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(toekn);
            }
            '=' => {
                let toekn = if self.advance_if_match('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(toekn);
            }
            '<' => {
                let toekn = if self.advance_if_match('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(toekn);
            }
            '>' => {
                let toekn = if self.advance_if_match('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(toekn);
            }
            '/' => {
                if self.advance_if_match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            '"' => self.string(),
            '\n' => self.line += 1,
            ' ' | '\r' | '\t' => {}
            c if Scanner::is_digit(c) => self.number(),
            c if Scanner::is_alpha(c) => self.identifier(),
            _ => self.error(self.line, "Unexpected character."),
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error(self.line, "Unterminated string.");
            return;
        }

        // The closing "
        self.advance();

        let value = self.source[(self.start + 1)..(self.current - 1)]
            .iter()
            .collect::<String>();
        self.add_token_value(TokenType::String, value.into());
    }

    fn number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        // Loof for fraction part.
        if self.peek() == '.' && Scanner::is_digit(self.peek_next()) {
            self.advance();

            while Scanner::is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let val = match value.parse::<f64>() {
            Ok(v) => v,
            Err(e) => {
                self.error(self.line, &format!("{:?}", e));
                return;
            }
        };

        self.add_token_value(TokenType::Number, Literal::Number(val));
    }

    fn error(&mut self, line: usize, message: &str) {
        if let Ok(mut lox) = self.lox.lock() {
            lox.error(line, message);
        }
    }

    fn identifier(&mut self) {
        while Scanner::is_alhpha_numberic(self.peek()) {
            self.advance();
        }

        let text = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let ty = TokenType::to_keyword(&text).unwrap_or(TokenType::Identifier);
        self.add_token(ty);
    }

    /// Only check for 10 Radix
    /// So leave character.is_digiht alone
    fn is_digit(c: char) -> bool {
        ('0'..='9').contains(&c)
    }

    fn is_alpha(c: char) -> bool {
        ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_'
    }

    fn is_alhpha_numberic(c: char) -> bool {
        Scanner::is_alpha(c) || Scanner::is_digit(c)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source[self.current + 1]
    }

    fn advance(&mut self) -> char {
        let idx = self.current;
        self.current += 1;
        self.source[idx]
    }

    fn advance_if_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn add_token(&mut self, ty: TokenType) {
        self.add_token_value(ty, Literal::Nil);
    }

    fn add_token_value(&mut self, ty: TokenType, literal: Literal) {
        let text = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        self.tokens.push(Token::new(ty, text, literal, self.line))
    }
}

use std::rc::Rc;

use super::{token::Token, TokenType};

pub struct Scanner {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: u32,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;
        if self.is_at_end() {
            return Token::new(TokenType::Eof, Default::default(), self.line);
        }

        match self.advance() {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            ';' => self.make_token(TokenType::Semicolon),
            '*' => self.make_token(TokenType::Star),
            '/' => self.make_token(TokenType::Slash),
            '!' => {
                let toekn = if self.advance_if_match('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.make_token(toekn)
            }
            '=' => {
                let toekn = if self.advance_if_match('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.make_token(toekn)
            }
            '<' => {
                let toekn = if self.advance_if_match('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.make_token(toekn)
            }
            '>' => {
                let toekn = if self.advance_if_match('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.make_token(toekn)
            }
            '"' => self.string(),
            c if Scanner::is_digit(c) => self.number(),
            c if Scanner::is_alpha(c) => self.identifier(),
            _ => self.error_token("Unexpected character."),
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }

                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => {}
            }
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        // The closing "
        self.advance();

        self.make_token(TokenType::String)
    }

    fn number(&mut self) -> Token {
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

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token {
        while Scanner::is_alhpha_numberic(self.peek()) {
            self.advance();
        }

        let text = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let ty = to_keyword(&text);
        self.make_token(ty)
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
        self.source.get(self.current).map_or(false, |c| *c == '\0')
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

    fn make_token(&mut self, ty: TokenType) -> Token {
        let text = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        Token::new(ty, Rc::new(text), self.line)
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            ty: TokenType::Error,
            str: Rc::new(message.to_string()),
            line: self.line,
        }
    }
}

fn to_keyword(str: &str) -> TokenType {
    match str {
        "and" => TokenType::And,
        "class" => TokenType::Class,
        "else" => TokenType::Else,
        "false" => TokenType::False,
        "for" => TokenType::For,
        "fn" => TokenType::Fn,
        "if" => TokenType::If,
        "nil" => TokenType::Nil,
        "or" => TokenType::Or,
        "print" => TokenType::Print,
        "return" => TokenType::Return,
        "super" => TokenType::Super,
        "this" => TokenType::This,
        "true" => TokenType::Ture,
        "var" => TokenType::Var,
        "while" => TokenType::While,
        "break" => TokenType::Break,
        _ => TokenType::Identifier,
    }
}

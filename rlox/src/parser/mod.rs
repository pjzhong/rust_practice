use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{
    expr::Expr,
    token::{Literal, Token, TokenType},
    Lox, LoxErr,
};

pub struct Parser {
    tokens: VecDeque<Token>,
    current: usize,
    lox: Arc<Mutex<Lox>>,
}

impl<T> From<LoxErr> for Result<T, LoxErr> {
    fn from(e: LoxErr) -> Self {
        Err(e)
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>, lox: Arc<Mutex<Lox>>) -> Self {
        Self {
            tokens: tokens.into(),
            lox,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Expr, LoxErr> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, LoxErr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, LoxErr> {
        let mut expr = self.comparison()?;

        while let Some(operator) = self.match_types(&[TokenType::BangEqual, TokenType::EqualEqual])
        {
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right))
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LoxErr> {
        let mut expr = self.term()?;
        while let Some(operator) = self.match_types(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LoxErr> {
        let mut expr = self.factor()?;

        while let Some(operator) = self.match_types(&[TokenType::Minus, TokenType::Plus]) {
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, LoxErr> {
        let mut expr = self.unary()?;
        while let Some(operator) = self.match_types(&[TokenType::Slash, TokenType::Star]) {
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxErr> {
        if let Some(operator) = self.match_types(&[TokenType::Bang, TokenType::Minus]) {
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, LoxErr> {
        if self.match_type(TokenType::False).is_some() {
            Ok(false.into())
        } else if self.match_type(TokenType::Ture).is_some() {
            Ok(true.into())
        } else if self.match_type(TokenType::Nil).is_some() {
            Ok(Literal::Nil.into())
        } else if let Some(token) = self.match_types(&[TokenType::Number, TokenType::String]) {
            Ok(Expr::Literal(token.value))
        } else if self.match_type(TokenType::LeftParen).is_some() {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression")?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            self.error("Expect expression.").into()
        }
    }

    fn consume(&mut self, ty: TokenType, message: &str) -> Result<(), LoxErr> {
        if self.check(ty) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    fn advance(&mut self) -> Option<Token> {
        if self.is_at_end() {
            None
        } else {
            self.tokens.pop_front()
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().toke_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn match_type(&mut self, ty: TokenType) -> Option<Token> {
        if self.check(ty) {
            self.advance()
        } else {
            None
        }
    }

    fn match_types(&mut self, types: &[TokenType]) -> Option<Token> {
        for ty in types {
            if self.check(*ty) {
                return self.advance();
            }
        }
        None
    }

    fn check(&self, ty: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().toke_type == ty
    }

    fn error(&mut self, message: &str) -> LoxErr {
        if let Ok(mut lox) = self.lox.lock() {
            let token = self.peek();
            lox.error_token(token, message)
        }
        LoxErr::ParseErr(message.to_string())
    }

    fn synchronize(&mut self) {
        while let Some(oper) = self.advance() {
            if oper.toke_type == TokenType::Semicolon {
                return;
            }
            if self.is_at_end() {
                return;
            }
            match self.peek().toke_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }
        }
    }
}
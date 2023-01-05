use crate::{token::{Token, TokenType}, expr::Expr};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {

    fn expression(&self) -> Expr {
        self.equality()
    }

    fn equality(&self) -> Expr {

    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        } 

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().toke_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
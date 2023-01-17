use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{
    ast::{Expr, Stmt},
    token::{Literal, Token, TokenType},
    Lox, LoxErr,
};

pub struct Parser {
    tokens: VecDeque<Token>,
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
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }
        statements
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let stmt = if self.match_type(TokenType::Var).is_some() {
            self.var_declaration()
        } else {
            self.statement()
        };

        match stmt {
            Ok(stmt) => Some(stmt),
            Err(e) => {
                if let Ok(mut lox) = self.lox.lock() {
                    lox.lox_error(e);
                }
                self.synchronize();
                None
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, LoxErr> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let initlaizer = if self.match_type(TokenType::Equal).is_some() {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after variable")?;
        Ok(Stmt::Var(name, initlaizer))
    }

    fn statement(&mut self) -> Result<Stmt, LoxErr> {
        match self.match_types(&[
            TokenType::Print,
            TokenType::LeftBrace,
            TokenType::If,
            TokenType::While,
            TokenType::For,
        ]) {
            Some(Token {
                toke_type: TokenType::If,
                ..
            }) => self.if_statment(),
            Some(Token {
                toke_type: TokenType::Print,
                ..
            }) => self.print_statement(),
            Some(Token {
                toke_type: TokenType::While,
                ..
            }) => self.while_statement(),
            Some(
                a @ Token {
                    toke_type: TokenType::For,
                    ..
                },
            ) => self.for_statement(a),
            Some(Token {
                toke_type: TokenType::LeftBrace,
                ..
            }) => self.block(),
            _ => self.expression_statment(),
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, LoxErr> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after print.")?;
        Ok(Stmt::Print(value))
    }

    fn while_statement(&mut self) -> Result<Stmt, LoxErr> {
        // I trying to use rust style
        //WhileStmt -> "while"  expression block
        let condition = self.expression()?;
        self.check_error(TokenType::LeftBrace, "While expect a block.")?;
        let body = self.statement()?;
        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn for_statement(&mut self, token: Token) -> Result<Stmt, LoxErr> {
        self.consume(TokenType::LeftParen, "For exepct ( after it")?;
        let initializer = match self.match_types(&[TokenType::Semicolon, TokenType::Var]) {
            Some(Token {
                toke_type: TokenType::Semicolon,
                ..
            }) => None,
            Some(Token {
                toke_type: TokenType::Var,
                ..
            }) => Some(self.var_declaration()?),
            _ => Some(self.expression_statment()?),
        };
        let condition = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::Semicolon, "expect ';' after loop condition")?;

        let increment = if self.check(TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::RightParen, "expect ')' after for clauses.")?;

        self.check_error(TokenType::LeftBrace, "for exepct a block")?;
        let mut stmts = match self.statement()? {
            Stmt::Block(stmts) => stmts,
            _ => return Err(self.error(&token, "for exepct a block")),
        };

        if let Some(increment) = increment {
            stmts.push(Stmt::Expression(increment));
        }

        let condition = if let Some(condtion) = condition {
            condtion
        } else {
            Expr::Literal(true.into())
        };

        let for_loop = if let Some(initializer) = initializer {
            let stmt_while = Stmt::While(condition, Box::new(Stmt::Block(stmts)));
            Stmt::Block(vec![initializer, stmt_while])
        } else {
            Stmt::While(condition, Box::new(Stmt::Block(stmts)))
        };

        Ok(for_loop)
    }

    fn block(&mut self) -> Result<Stmt, LoxErr> {
        let mut statments = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statments.push(stmt);
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(Stmt::Block(statments))
    }

    fn if_statment(&mut self) -> Result<Stmt, LoxErr> {
        // I trying to use rust style
        //ifStmt -> "if"  expression block
        //          ( "else" block )?
        let condition = self.expression()?;
        self.check_error(TokenType::LeftBrace, "expect a block after if condition")?;
        let the_branch = self.statement()?;

        let else_branch = if self.match_type(TokenType::Else).is_some() {
            if self.check(TokenType::If) {
                Some(self.statement()?)
            } else {
                self.check_error(TokenType::LeftBrace, "expect a block after else")?;
                Some(self.statement()?)
            }
        } else {
            None
        };

        Ok(Stmt::If(
            condition,
            Box::new(the_branch),
            else_branch.map(Box::new),
        ))
    }

    fn expression_statment(&mut self) -> Result<Stmt, LoxErr> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(value))
    }

    fn expression(&mut self) -> Result<Expr, LoxErr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, LoxErr> {
        let expr = self.or()?;

        if let Some(equal) = self.match_type(TokenType::Equal) {
            let value = self.assignment()?;

            match expr {
                Expr::Variable(name) => return Ok(Expr::Assign(name, Box::new(value))),
                _ => self.report_error(&equal, "Invalid assignment target."),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, LoxErr> {
        let mut expr = self.and()?;

        while let Some(token) = self.match_type(TokenType::Or) {
            let right = self.and()?;
            expr = Expr::Logical(Box::new(expr), token, Box::new(right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, LoxErr> {
        let mut expr = self.equality()?;

        while let Some(token) = self.match_type(TokenType::And) {
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), token, Box::new(right));
        }

        Ok(expr)
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
        match self.advance() {
            Some(token) => match token.toke_type {
                TokenType::False => Ok(false.into()),
                TokenType::Ture => Ok(true.into()),
                TokenType::Nil => Ok(Literal::Nil.into()),
                TokenType::Identifier => Ok(Expr::Variable(token)),
                TokenType::Number | TokenType::String => Ok(Expr::Literal(token.value)),
                TokenType::LeftParen => {
                    let expr = self.expression()?;
                    self.consume(TokenType::RightParen, "Expect ')' after expression")?;
                    Ok(Expr::Grouping(Box::new(expr)))
                }
                _ => {
                    let err = Err(self.error(&token, "Expect expression."));
                    self.tokens.push_front(token);
                    err
                }
            },
            None => Err(LoxErr::ParseErr(
                0,
                TokenType::Eof,
                "unknown".to_string().into(),
                "Unexpected end, Expect expression.".to_string(),
            )),
        }
    }

    fn consume(&mut self, ty: TokenType, message: &str) -> Result<Token, LoxErr> {
        if let Some(token) = self.advance() {
            if token.toke_type == ty {
                Ok(token)
            } else {
                let err = Err(self.error(&token, message));
                self.tokens.push_front(token);
                err
            }
        } else {
            Err(LoxErr::ParseErr(
                0,
                TokenType::Eof,
                "unknown".to_string().into(),
                format!("{}, {}", "Unexpected end", message),
            ))
        }
    }

    fn check_error(&mut self, ty: TokenType, message: &str) -> Result<(), LoxErr> {
        if let Some(token) = self.peek() {
            if token.toke_type == ty {
                Ok(())
            } else {
                Err(self.error(token, message))
            }
        } else {
            Err(LoxErr::ParseErr(
                0,
                TokenType::Eof,
                "unknown".to_string().into(),
                format!("{}, {}", "Unexpected end", message),
            ))
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn is_at_end(&self) -> bool {
        self.peek().map_or(true, |t| t.toke_type == TokenType::Eof)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
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
        self.peek().map_or(false, |t| t.toke_type == ty)
    }

    fn error(&self, token: &Token, message: &str) -> LoxErr {
        LoxErr::ParseErr(
            token.line,
            token.toke_type,
            token.lexeme.clone(),
            message.to_string(),
        )
    }

    fn report_error(&mut self, token: &Token, message: &str) {
        if let Ok(mut lox) = self.lox.lock() {
            lox.error(token.line, message)
        }
    }

    fn synchronize(&mut self) {
        while let Some(oper) = self.advance() {
            if oper.toke_type == TokenType::Semicolon {
                return;
            }

            match self.peek() {
                Some(token) => match token.toke_type {
                    TokenType::Class
                    | TokenType::Fun
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return => return,
                    _ => {}
                },
                None => return,
            }
        }
    }
}

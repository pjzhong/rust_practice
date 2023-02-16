use std::rc::Rc;

pub struct Token {
    pub ty: TokenType,
    pub str: Rc<String>,
    pub line: u32,
}

impl Token {
    pub fn new(ty: TokenType, str: Rc<String>, line: u32) -> Self {
        Self { ty, str, line }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fn,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    Ture,
    Var,
    While,
    Break,

    Eof,
    Error,
}

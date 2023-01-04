use std::fmt::{Debug, Display};

pub struct Token {
    pub toke_type: TokenType,
    pub lexeme: String,
    pub value: Option<Literal>,
    pub line: usize,
}

impl Token {
    pub fn new<Str: Into<String>>(
        toke_type: TokenType,
        lexeme: Str,
        value: Option<Literal>,
        line: usize,
    ) -> Self {
        Self {
            toke_type,
            lexeme: lexeme.into(),
            value,
            line,
        }
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("toke_type", &self.toke_type)
            .field("lexeme", &self.lexeme)
            .field("value", &self.value)
            .finish()
    }
}

#[derive(Debug)]
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
    Fun,
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

    Eof,
}

#[derive(Debug)]
pub enum Literal {
    String(String),
    Number(f64),
}

///简化代码编写，不然这种包装写法太长了
impl From<f64> for Literal {
    fn from(a: f64) -> Self {
        Literal::Number(a)
    }
}

impl From<String> for Literal {
    fn from(a: String) -> Self {
        Literal::String(a)
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(str) => write!(f, "{}", str),
            Literal::Number(num) => write!(f, "{}", num),
        }
    }
}

impl TokenType {
    pub fn to_keyword(str: &str) -> Option<TokenType> {
        match str {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "for" => Some(TokenType::For),
            "fun" => Some(TokenType::Fun),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::Ture),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }
}
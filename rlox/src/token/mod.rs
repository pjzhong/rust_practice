use std::{
    fmt::{Debug, Display},
    hash::Hash,
    rc::Rc,
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Token {
    pub toke_type: TokenType,
    pub lexeme: Rc<String>,
    pub value: Literal,
    pub line: usize,
}

impl Token {
    pub fn new(toke_type: TokenType, lexeme: String, value: Literal, line: usize) -> Self {
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
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(Rc<String>),
    Number(f64),
    Bool(bool),
    Nil,
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}
// Why this would Work
impl Eq for Literal {}

impl Hash for Literal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Literal::String(str) => str.hash(state),
            Literal::Number(num) => {
                let bites = num.to_bits();
                let bites = bites ^ (bites >> 32);
                state.write_u64(bites)
            }
            Literal::Bool(a) => a.hash(state),
            Literal::Nil => state.write_i8(0),
        }
    }
}

///简化代码编写，不然这种包装写法太长了
impl From<f64> for Literal {
    fn from(a: f64) -> Self {
        Literal::Number(a)
    }
}

impl From<String> for Literal {
    fn from(a: String) -> Self {
        Literal::String(Rc::new(a))
    }
}

impl From<bool> for Literal {
    fn from(a: bool) -> Self {
        Literal::Bool(a)
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(str) => write!(f, "{}", str),
            Literal::Number(num) => write!(f, "{}", num),
            Literal::Bool(bol) => write!(f, "{}", bol),
            Literal::Nil => write!(f, "null"),
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
            "fn" => Some(TokenType::Fn),
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
            "break" => Some(TokenType::Break),
            _ => None,
        }
    }
}

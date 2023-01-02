use std::fmt::Debug;

pub struct Token {
    pub toke_type: TokenType,
    pub lexeme: String,
    pub value: Option<TokenValue>,
    pub line: usize,
}

impl Token {
    pub fn new(
        toke_type: TokenType,
        lexeme: String,
        value: Option<TokenValue>,
        line: usize,
    ) -> Self {
        Self {
            toke_type,
            lexeme,
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
    IDENTIFIER,
    STRING,
    NUMBER,

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
pub enum TokenValue {
    Literal(String),
    Number(f64),
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

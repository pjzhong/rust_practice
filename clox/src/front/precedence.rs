use super::{compile::ParseFn, Compiler, TokenType};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

pub struct ParseRule {
    pub prefix: ParseFn,
    pub infix: ParseFn,
    pub precedence: Precedence,
}

impl Precedence {
    pub fn heigher(&self) -> Self {
        match self {
            Precedence::None => Self::Assignment,
            Precedence::Assignment => Self::Or,
            Precedence::Or => Self::And,
            Precedence::And => Self::Equality,
            Precedence::Equality => Self::Comparison,
            Precedence::Comparison => Self::Term,
            Precedence::Term => Self::Factor,
            Precedence::Factor => Self::Unary,
            Precedence::Unary => Self::Call,
            Precedence::Call => Self::Primary,
            Precedence::Primary => Self::Primary,
        }
    }
}

fn nothing(_: &mut Compiler, _: bool) {}

pub fn get_rule(ty: TokenType) -> ParseRule {
    const LEFT_PAREN: ParseRule = ParseRule {
        prefix: Compiler::grouping,
        infix: Compiler::call,
        precedence: Precedence::Call,
    };
    const MINUS: ParseRule = ParseRule {
        prefix: Compiler::unary,
        infix: Compiler::binary,
        precedence: Precedence::Term,
    };
    const PLUS: ParseRule = ParseRule {
        prefix: nothing,
        infix: Compiler::binary,
        precedence: Precedence::Term,
    };
    const SLASH: ParseRule = ParseRule {
        prefix: nothing,
        infix: Compiler::binary,
        precedence: Precedence::Factor,
    };
    const STAR: ParseRule = ParseRule {
        prefix: nothing,
        infix: Compiler::binary,
        precedence: Precedence::Factor,
    };
    const NUMBER: ParseRule = ParseRule {
        prefix: Compiler::number,
        infix: nothing,
        precedence: Precedence::None,
    };
    const BOOL: ParseRule = ParseRule {
        prefix: Compiler::literal,
        infix: nothing,
        precedence: Precedence::None,
    };
    const NIL: ParseRule = ParseRule {
        prefix: Compiler::literal,
        infix: nothing,
        precedence: Precedence::None,
    };
    const BANG: ParseRule = ParseRule {
        prefix: Compiler::unary,
        infix: nothing,
        precedence: Precedence::None,
    };
    const BANG_EQUAL: ParseRule = ParseRule {
        prefix: nothing,
        infix: Compiler::binary,
        precedence: Precedence::Equality,
    };
    const EQUAL_EQUAL: ParseRule = ParseRule {
        prefix: nothing,
        infix: Compiler::binary,
        precedence: Precedence::Equality,
    };
    const GREATER: ParseRule = ParseRule {
        prefix: nothing,
        infix: Compiler::binary,
        precedence: Precedence::Comparison,
    };
    const GREATER_EQUAL: ParseRule = ParseRule {
        prefix: nothing,
        infix: Compiler::binary,
        precedence: Precedence::Comparison,
    };
    const LESS: ParseRule = ParseRule {
        prefix: nothing,
        infix: Compiler::binary,
        precedence: Precedence::Comparison,
    };
    const LESS_EQUAL: ParseRule = ParseRule {
        prefix: nothing,
        infix: Compiler::binary,
        precedence: Precedence::Comparison,
    };
    const STRING: ParseRule = ParseRule {
        prefix: Compiler::string,
        infix: nothing,
        precedence: Precedence::None,
    };
    const IDENTIFIER: ParseRule = ParseRule {
        prefix: Compiler::varaible,
        infix: nothing,
        precedence: Precedence::None,
    };
    const AND: ParseRule = ParseRule {
        prefix: nothing,
        infix: Compiler::and,
        precedence: Precedence::And,
    };
    const OR: ParseRule = ParseRule {
        prefix: nothing,
        infix: Compiler::or,
        precedence: Precedence::And,
    };
    const NONE: ParseRule = ParseRule {
        prefix: nothing,
        infix: nothing,
        precedence: Precedence::None,
    };
    match ty {
        TokenType::LeftParen => LEFT_PAREN,
        TokenType::Minus => MINUS,
        TokenType::Plus => PLUS,
        TokenType::Slash => SLASH,
        TokenType::Star => STAR,
        TokenType::Number => NUMBER,
        TokenType::Ture => BOOL,
        TokenType::False => BOOL,
        TokenType::Nil => NIL,
        TokenType::Bang => BANG,
        TokenType::BangEqual => BANG_EQUAL,
        TokenType::EqualEqual => EQUAL_EQUAL,
        TokenType::Greater => GREATER,
        TokenType::GreaterEqual => GREATER_EQUAL,
        TokenType::Less => LESS,
        TokenType::LessEqual => LESS_EQUAL,
        TokenType::Identifier => IDENTIFIER,
        TokenType::String => STRING,
        TokenType::And => AND,
        TokenType::Or => OR,
        _ => NONE,
    }
}

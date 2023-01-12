use crate::token::TokenType;

#[derive(Debug)]
pub enum LoxErr {
    ParseErr(usize, TokenType, String, String),
    RunTimeErr(Option<usize>, String),
}

use std::sync::Arc;

use crate::token::TokenType;

#[derive(Debug)]
pub enum LoxErr {
    ParseErr(usize, TokenType, Arc<String>, String),
    RunTimeErr(Option<usize>, String),
    BreakOutSideLoop,
}

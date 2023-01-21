use std::rc::Rc;

use crate::token::TokenType;

#[derive(Debug)]
pub enum LoxErr {
    ParseErr(usize, TokenType, Rc<String>, String),
    RunTimeErr(Option<usize>, String),
    BreakOutSideLoop,
}

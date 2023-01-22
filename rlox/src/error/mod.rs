use std::rc::Rc;

use crate::{interpreter::LoxValue, token::TokenType};

#[derive(Debug)]
pub enum LoxErr {
    ParseErr(usize, TokenType, Rc<String>, String),
    RunTimeErr(Option<usize>, String),
    Return(LoxValue),
    BreakOutSideLoop,
}

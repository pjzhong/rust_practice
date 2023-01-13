use crate::{interpreter::LoxValue, token::Token, LoxErr};
use std::{collections::HashMap, fmt::format, sync::Arc};

#[derive(Debug, Default)]
pub struct Environment {
    values: HashMap<Arc<String>, LoxValue>,
}

impl Environment {
    pub fn define(&mut self, name: Arc<String>, value: LoxValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, token: &Token) -> Result<LoxValue, LoxErr> {
        let name = &token.lexeme;
        let line = token.line;
        self.values.get(name).map_or_else(
            || {
                Err(LoxErr::RunTimeErr(
                    Some(line),
                    format!("Undefined variable '{}'", name),
                ))
            },
            |v| Ok(v.clone()),
        )
    }

    pub fn assign(&mut self, token: &Token, value: &LoxValue) -> Result<(), LoxErr> {
        if let Some(val) = self.values.get_mut(&token.lexeme) {
            *val = value.clone();
            Ok(())
        } else {
            Err(LoxErr::RunTimeErr(
                Some(token.line),
                format!("Undefined variable '{}'.", token.lexeme),
            ))
        }
    }
}

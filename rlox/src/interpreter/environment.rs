use std::collections::HashMap;
use crate::interpreter::LoxValue;

#[derive(Debug, Default)]
struct Environment {
    values: HashMap<String, LoxValue>,
}

impl Environment {
    
    fn define(&mut self, name: String, value: LoxValue) {
        self.values.insert(name, value);
    }
}
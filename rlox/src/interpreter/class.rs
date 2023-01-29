use std::{fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub struct Class {
    name: Rc<String>,
}

impl From<Rc<String>> for Class {
    fn from(name: Rc<String>) -> Self {
        Self { name }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "class {}", self.name)
    }
}

#[derive(Debug)]
pub struct Instance {
    klass: Rc<Class>,
}

impl Instance {
    pub fn new(klass: Rc<Class>) -> Self {
        Self { klass }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.klass.name)
    }
}

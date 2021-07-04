use core::{result, fmt};
use std::io;
use std::fmt::Formatter;

pub type Result<T> = result::Result<T, CliErr>;

#[derive(Debug)]
pub enum CliErr {
    IoError(io::Error),
    BsonDeError(bson::de::Error),
    BsonSerError(bson::ser::Error),
    KeyNotFound,
}

impl fmt::Display for CliErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            CliErr::KeyNotFound => write!(f, "Key not found"),
            _ => write!(f, "Parse error: {}", self),
        }
    }
}

impl From<io::Error> for CliErr {
    fn from(err: io::Error) -> Self {
        CliErr::IoError(err)
    }
}

impl From<bson::de::Error> for CliErr {
    fn from(err: bson::de::Error) -> Self {
        CliErr::BsonDeError(err)
    }
}

impl From<bson::ser::Error> for CliErr {
    fn from(err: bson::ser::Error) -> Self {
        CliErr::BsonSerError(err)
    }
}




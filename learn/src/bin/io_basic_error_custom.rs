use core::{fmt, num};
use std::fmt::Formatter;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{error, io};

#[derive(Debug)]
enum CliError {
    Io(io::Error),
    Parse(num::ParseIntError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            CliError::Io(ref err) => write!(f, "IO error: {}", err),
            CliError::Parse(ref err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl error::Error for CliError {
}

fn file_double<P: AsRef<Path>>(file_path: P) -> Result<i32, CliError> {
    let mut file = File::open(file_path).map_err(CliError::Io)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(CliError::Io)?;
    let n = contents.trim().parse::<i32>().map_err(CliError::Parse)?;

    Ok(2 * n)
}

fn main() {
    match file_double("D:\\work\\rust_practice\\foobar") {
        Ok(n) => println!("{}", n),
        Err(err) => println!("{}", err),
    }
}

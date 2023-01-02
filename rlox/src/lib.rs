mod scanner;
mod token;

pub use scanner::Scanner;

#[derive(Default)]
pub struct Lox {
    pub has_error: bool,
}

impl Lox {
    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message)
    }

    pub fn report(&mut self, line: usize, location: &str, message: &str) {
        println!("[line {:?}] Error {:?}:{:?}", line, location, message);
        self.has_error = true;
    }
}

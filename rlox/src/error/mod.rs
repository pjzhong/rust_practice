#[derive(Debug)]
pub enum LoxErr {
    ParseErr(String),
    RunTimeErr(Option<usize>, String),
}

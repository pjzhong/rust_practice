#[derive(Debug, PartialEq)]
pub enum ConfigChange {
    General(Box<Config>)
}
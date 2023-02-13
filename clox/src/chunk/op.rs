pub enum OpCode {
    Return,
    Constant,
    Unknown(u8),
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        return match byte {
            0 => Self::Return,
            1 => Self::Constant,
            _ => Self::Unknown(byte),
        };
    }
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> Self {
        match op {
            OpCode::Return => 0,
            OpCode::Constant => 1,
            OpCode::Unknown(a) => a,
        }
    }
}

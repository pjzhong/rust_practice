pub enum OpCode {
    Return,
    Constant,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Unknown(u8),
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Self::Return,
            1 => Self::Constant,
            2 => Self::Negate,
            3 => Self::Add,
            4 => Self::Subtract,
            5 => Self::Multiply,
            6 => Self::Divide,
            _ => Self::Unknown(byte),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> Self {
        match op {
            OpCode::Return => 0,
            OpCode::Constant => 1,
            OpCode::Negate => 2,
            OpCode::Add => 3,
            OpCode::Subtract => 4,
            OpCode::Multiply => 5,
            OpCode::Divide => 6,
            OpCode::Unknown(a) => a,
        }
    }
}

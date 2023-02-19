#[derive(Debug)]
pub enum OpCode {
    Return,
    Constant,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Nil,
    True,
    False,
    Bang,
    Equal,
    Greater,
    Less,
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
            7 => Self::Nil,
            8 => Self::True,
            9 => Self::False,
            10 => Self::Bang,
            11 => Self::Equal,
            12 => Self::Greater,
            13 => Self::Less,
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
            OpCode::Nil => 7,
            OpCode::True => 8,
            OpCode::False => 9,
            OpCode::Bang => 10,
            OpCode::Equal => 11,
            OpCode::Greater => 12,
            OpCode::Less => 13,
            OpCode::Unknown(a) => a,
        }
    }
}

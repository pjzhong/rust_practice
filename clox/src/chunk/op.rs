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
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    Jump,
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
            14 => Self::Print,
            15 => Self::Pop,
            16 => Self::DefineGlobal,
            17 => Self::GetGlobal,
            18 => Self::SetGlobal,
            19 => Self::GetLocal,
            20 => Self::SetLocal,
            21 => Self::JumpIfFalse,
            22 => Self::Jump,
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
            OpCode::Print => 14,
            OpCode::Pop => 15,
            OpCode::DefineGlobal => 16,
            OpCode::GetGlobal => 17,
            OpCode::SetGlobal => 18,
            OpCode::GetLocal => 19,
            OpCode::SetLocal => 20,
            OpCode::JumpIfFalse => 21,
            OpCode::Jump => 22,
            OpCode::Unknown(a) => a,
        }
    }
}

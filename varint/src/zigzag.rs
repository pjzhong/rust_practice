/// A trait for enabling zig-zag encoding of various values
pub trait ZigZag<T> {
    fn zigzag(&self) -> T;
}

impl ZigZag<u8> for i8 {
    /// Encodes this iu as zigzagged u8
    fn zigzag(&self) -> u8 {
        //左移一位
        // (1 1 1 1 1 1 1 1 << 1) == (1 1 1 1 1 1 1 0)
        //右移动7位
        // (1 0 0 0 0 0 0 0 >> 7) == (0 0 0 0 0 0 0 1)
        ((self << 1) ^ (self >> 7)) as u8
    }
}

impl ZigZag<i8> for u8 {
    fn zigzag(&self) -> i8 {
        //i8的逆序
        //内容右移一位
        //^
        //只保留最低位,如果没有就是零，有就是-1(1 1 1 1 1 1 1 1)
        ((self >> 1) as i8) ^ (-((self & 1) as i8))
    }
}

impl ZigZag<u16> for i16 {
    /// Encodes this iu as zigzagged u8
    fn zigzag(&self) -> u16 {
        //左移一位
        // (1 1 1 1 1 1 1 1 << 1) == (1 1 1 1 1 1 1 0)
        //右移动7位
        // (1 0 0 0 0 0 0 0 >> 7) == (0 0 0 0 0 0 0 1)
        ((self << 1) ^ (self >> 15)) as u16
    }
}

impl ZigZag<i16> for u16 {
    fn zigzag(&self) -> i16 {
        //i8的逆序
        //内容右移一位
        //^
        //只保留最低位,如果没有就是零，有就是-1(1 1 1 1 1 1 1 1)
        ((self >> 1) as i16) ^ (-((self & 1) as i16))
    }
}

impl ZigZag<u32> for i32 {
    /// Encodes this iu as zigzagged u8
    fn zigzag(&self) -> u32 {
        //左移一位
        // (1 1 1 1 1 1 1 1 << 1) == (1 1 1 1 1 1 1 0)
        //右移动7位
        // (1 0 0 0 0 0 0 0 >> 7) == (0 0 0 0 0 0 0 1)
        ((self << 1) ^ (self >> 31)) as u32
    }
}

impl ZigZag<i32> for u32 {
    fn zigzag(&self) -> i32 {
        //i8的逆序
        //内容右移一位
        //^
        //只保留最低位,如果没有就是零，有就是-1(1 1 1 1 1 1 1 1)
        ((self >> 1) as i32) ^ (-((self & 1) as i32))
    }
}

impl ZigZag<u64> for i64 {
    /// Encodes this iu as zigzagged u8
    fn zigzag(&self) -> u64 {
        //左移一位
        // (1 1 1 1 1 1 1 1 << 1) == (1 1 1 1 1 1 1 0)
        //右移动7位
        // (1 0 0 0 0 0 0 0 >> 7) == (0 0 0 0 0 0 0 1)
        ((self << 1) ^ (self >> 63)) as u64
    }
}

impl ZigZag<i64> for u64 {
    fn zigzag(&self) -> i64 {
        //i8的逆序
        //内容右移一位
        //^
        //只保留最低位,如果没有就是零，有就是-1(1 1 1 1 1 1 1 1)
        ((self >> 1) as i64) ^ (-((self & 1) as i64))
    }
}

#[cfg(test)]
mod tests {

    use super::ZigZag;

    #[test]
    fn test_u8_i8_zigzag() {
        assert_eq!(0i8, 0u8.zigzag());

        assert_eq!(-1i8, 1u8.zigzag());

        assert_eq!(0u8, 0i8.zigzag());

        assert_eq!(1u8, (-1i8).zigzag());

        assert_eq!(2u8, 1i8.zigzag());
    }

    #[test]
    fn test_u16_i16_zigzag() {
        assert_eq!(0i16, 0u16.zigzag());

        assert_eq!(-1i16, 1u16.zigzag());

        assert_eq!(0u16, 0i16.zigzag());

        assert_eq!(1u16, (-1i16).zigzag());

        assert_eq!(2u16, 1i16.zigzag());
    }

    #[test]
    fn test_u32_i32_zigzag() {
        assert_eq!(0i32, 0u32.zigzag());

        assert_eq!(-1i32, 1u32.zigzag());

        assert_eq!(0u32, 0i32.zigzag());

        assert_eq!(1u32, (-1i32).zigzag());

        assert_eq!(2u32, 1i32.zigzag());
    }

    #[test]
    fn test_u64_i64_zigzag() {
        assert_eq!(0i64, 0u64.zigzag());

        assert_eq!(-1i64, 1u64.zigzag());

        assert_eq!(0u64, 0i64.zigzag());

        assert_eq!(1u64, (-1i64).zigzag());

        assert_eq!(2u64, 1i64.zigzag());
    }
}

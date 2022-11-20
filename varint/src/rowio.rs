use std::{
    io::{Error, Write, ErrorKind, Read}, vec,
};

use crate::zigzag::ZigZag;

pub trait VarintRead: Read {
    fn read_varint_i32(&mut self) -> Result<i32, Error> {
        match self.read_varint_u64() {
            Ok(value) => {
                Ok((value as u32).zigzag())
            }
            Err(err) => {
                Err(err)
            }
        }
    }

    fn read_varint_u32(&mut self) -> Result<u32, Error> {
        match self.read_varint_u64() {
            Ok(value) => {
                Ok(value as u32)
            }
            Err(err) => {
                Err(err)
            }
        }
    }

    fn read_varint_i64(&mut self) -> Result<i64, Error> {
        match self.read_varint_u64() {
            Ok(value) => {
                Ok(value.zigzag())
            }
            Err(err) => {
                Err(err)
            }
        }
    }

    fn read_varint_u64(&mut self) -> Result<u64, Error> {
        let mut decoded_value: u64 = 0;
        let mut raw_buffer = vec![0u8; 1];

        for shift_amount in (0..64).step_by(7) {
            let next_byte = match self.read(&mut raw_buffer) {
                Ok(count) => {
                    if count == 1 {
                        raw_buffer[0]
                    } else {
                        return Err(Error::new(
                            ErrorKind::Other,
                            "Could not read one byte (end of stream?)",
                        ));
                    }
                }
                Err(error) => {
                    return Err(error);
                }
            };

            decoded_value |= ((next_byte & 0x7F) as u64) << shift_amount;
            if (next_byte & 0x80) == 0 {
                return Ok(decoded_value);
            }
        }

        return Err(Error::new(
            ErrorKind::Other,
            "UnknowError, Could not decode rawu64",
        ));
    }
}

pub trait VarintWrite: Write {

    fn write_varint_i32(&mut self, value: i32) -> Result<(), Error> {
        self.write_varint_u32(value.zigzag())
    }


    fn write_varint_u32(&mut self, value: u32) -> Result<(), Error> {
        let mut value = value;
        if value == 0 {
            return self.write_all(&[0]);
        } else {
            while value >= 0x80 {
                let next_byte: u8 = ((value & 0x7f) as u8) | 0x80;
                value = value >> 7;
               
                let temp = self.write_all(&[next_byte]);
                if temp.is_err() {
                    return temp;
                }
            }

            return self.write_all(&[(value & 0x7f) as u8])
        }
    }

    fn write_varint_i64(&mut self, value: i64) -> Result<(), Error> {
        self.write_varint_u64(value.zigzag())
    }

    
    fn write_varint_u64(&mut self, value: u64) -> Result<(), Error> {
        let mut value = value;
        if value == 0 {
            return self.write_all(&[0]);
        } else {
            while value >= 0x80 {
                let next_byte: u8 = ((value & 0x7f) as u8) | 0x80;
                value = value >> 7;
               
                let temp = self.write_all(&[next_byte]);
                if temp.is_err() {
                    return temp;
                }
            }

            return self.write_all(&[(value & 0x7f) as u8])
        }
    }
}

impl VarintRead for ::std::io::Cursor<Vec<u8>> { }
impl VarintWrite for ::std::io::Cursor<Vec<u8>> { }

#[cfg(test)]
mod test {

    use std::io::Cursor;
    use super::{VarintRead, VarintWrite};

    #[test]
    fn test_read_write_varint_u32() {
        let mut vec = Cursor::new(vec![0u8;0]);

        assert!(vec.write_varint_u32(15).is_ok());
        assert_eq!(1, vec.position());
        assert!(vec.write_varint_u32(0).is_ok());
        assert_eq!(2, vec.position());
        assert!(vec.write_varint_u32(123412341).is_ok());
        assert!(vec.write_varint_u32(5545453).is_ok());
        assert!(vec.write_varint_u32(u32::MAX).is_ok());

        vec.set_position(0);

        assert_eq!(15, vec.read_varint_u32().unwrap());
        assert_eq!(0, vec.read_varint_u32().unwrap());
        assert_eq!(123412341, vec.read_varint_u32().unwrap());
        assert_eq!(5545453, vec.read_varint_u32().unwrap());
        assert_eq!(u32::MAX, vec.read_varint_u32().unwrap());
    }

    #[test]
    fn test_read_write_varint_i32() {
        let mut vec = Cursor::new(vec![0u8;0]);

        assert!(vec.write_varint_i32(15).is_ok());
        assert!(vec.write_varint_i32(0).is_ok());
        assert!(vec.write_varint_i32(123412341).is_ok());
        assert!(vec.write_varint_i32(-5545453).is_ok());
        assert!(vec.write_varint_i32(i32::MIN).is_ok());
        assert!(vec.write_varint_i32(i32::MAX).is_ok());

        vec.set_position(0);

        assert_eq!(15, vec.read_varint_i32().unwrap());
        assert_eq!(0, vec.read_varint_i32().unwrap());
        assert_eq!(123412341, vec.read_varint_i32().unwrap());
        assert_eq!(-5545453, vec.read_varint_i32().unwrap());
        assert_eq!(i32::MIN, vec.read_varint_i32().unwrap());
        assert_eq!(i32::MAX, vec.read_varint_i32().unwrap());
    }


    #[test]
    fn test_read_write_varint_u64() {
        let mut vec = Cursor::new(vec![0u8;0]);

        assert!(vec.write_varint_u64(15).is_ok());
        assert!(vec.write_varint_u64(0).is_ok());
        assert!(vec.write_varint_u64(123412341).is_ok());
        assert!(vec.write_varint_u64(5545453).is_ok());
        assert!(vec.write_varint_u64(u64::MAX).is_ok());

        vec.set_position(0);

        assert_eq!(15, vec.read_varint_u64().unwrap());
        assert_eq!(0, vec.read_varint_u64().unwrap());
        assert_eq!(123412341, vec.read_varint_u64().unwrap());
        assert_eq!(5545453, vec.read_varint_u64().unwrap());
        assert_eq!(u64::MAX, vec.read_varint_u64().unwrap());
    }

    #[test]
    fn test_read_write_varint_i64() {
        let mut vec = Cursor::new(vec![0u8;0]);

        assert!(vec.write_varint_i64(15).is_ok());
        assert_eq!(1, vec.position());
        assert!(vec.write_varint_i64(0).is_ok());
        assert_eq!(2, vec.position());
        assert!(vec.write_varint_i64(123412341).is_ok());
        assert_eq!(6, vec.position());
        assert!(vec.write_varint_i64(123412341234123441).is_ok());
        assert!(vec.write_varint_i64(-5545453).is_ok());
        assert!(vec.write_varint_i64(-55454534562345233).is_ok());
        assert!(vec.write_varint_i64(i64::MAX).is_ok());
        assert!(vec.write_varint_i64(i64::MIN).is_ok());


        vec.set_position(0);

        assert_eq!(15, vec.read_varint_i64().unwrap());
        assert_eq!(0, vec.read_varint_i64().unwrap());
        assert_eq!(123412341, vec.read_varint_i64().unwrap());
        assert_eq!(123412341234123441, vec.read_varint_i64().unwrap());
        assert_eq!(-5545453, vec.read_varint_i64().unwrap());
        assert_eq!(-55454534562345233, vec.read_varint_i64().unwrap());
        assert_eq!(i64::MAX, vec.read_varint_i64().unwrap());
        assert_eq!(i64::MIN, vec.read_varint_i64().unwrap());
    }
}
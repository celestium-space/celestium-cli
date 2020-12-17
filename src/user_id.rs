use crate::serialize::Serialize;
use std::fmt;

pub struct UserId {
    is_continuation: bool,
    is_magic: bool,
    value: u16,
}

impl UserId {
    pub fn new(is_continuation: bool, is_magic: bool, value: u16) -> UserId {
        UserId {
            is_continuation: is_continuation,
            is_magic: is_magic,
            value: value,
        }
    }

    pub fn is_continuation(&self) -> bool {
        self.is_continuation
    }

    pub fn is_magic(&self) -> bool {
        self.is_magic
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BID:{}", self.value)
    }
}

impl Serialize for UserId {
    fn from_serialized(data: &[u8], i: &mut usize) -> Result<Box<UserId>, String> {
        let uid = UserId {
            is_continuation: data[*i] & 0x80 > 0,
            is_magic: data[*i] == 0xef && data[*i + 1] == 0xff,
            value: (((data[*i] & 0x7f) as u16) << 8) + data[*i + 1] as u16,
        };
        *i += 2;
        return Ok(Box::new(uid));
    }
    fn serialize_into(&mut self, buffer: &mut [u8], i: &mut usize) -> Result<usize, String> {
        if self.is_magic {
            buffer[*i] = 0xef;
            buffer[*i + 1] = 0xff;
            *i += 2;
            return Ok(2);
        }
        let mut first_byte = (self.value >> 8) as u8;
        if self.is_continuation {
            first_byte ^= 0x80;
        }
        buffer[*i] = first_byte;
        buffer[*i + 1] = (self.value & 0xff) as u8;
        *i += 2;
        return Ok(2);
    }
    // fn my_serialize(&mut self) -> Result<Vec<u8>, String> {
    //     if self.is_magic {
    //         return Ok(vec![0xef, 0xff]);
    //     }
    //     let mut first_byte = (self.value >> 8) as u8;
    //     if self.is_continuation {
    //         first_byte ^= 0x80;
    //     }
    //     return Ok(vec![first_byte, (self.value & 0xff) as u8]);
    // }

    fn serialized_len(&self) -> Result<usize, String> {
        Ok(2)
    }
}

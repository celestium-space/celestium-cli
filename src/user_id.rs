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
    fn from_serialized(data: &[u8]) -> Result<Box<UserId>, String> {
        Ok(Box::new(UserId {
            is_continuation: data[0] & 0x80 > 0,
            is_magic: data[0] == 0xef && data[1] == 0xff,
            value: (((data[0] & 0x7f) as u16) << 8) + data[1] as u16,
        }))
    }
    fn serialize_into(&mut self, buffer: &mut [u8]) -> Result<Vec<u8>, String> {
        match &self.serialize() {
            Ok(s) => {
                buffer[0] = s[0];
                buffer[1] = s[1];
                return Ok(vec![s[0], s[1]]);
            }
            Err(e) => Err(e.to_string()),
        }
    }
    fn serialize(&mut self) -> Result<Vec<u8>, String> {
        if self.is_magic {
            return Ok(vec![0xef, 0xff]);
        }
        let mut first_byte = (self.value >> 8) as u8;
        if self.is_continuation {
            first_byte ^= 0x80;
        }
        return Ok(vec![first_byte, (self.value & 0xff) as u8]);
    }

    fn serialized_len(&self) -> Result<usize, String> {
        Ok(2)
    }
}

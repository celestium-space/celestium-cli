use crate::serialize::Serialize;
use std::fmt;

pub struct BlockId {
    is_continuation: bool,
    is_magic: bool,
    value: u16,
}

impl BlockId {
    pub fn new(is_continuation: bool, is_magic: bool, value: u16) -> BlockId {
        BlockId {
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

    pub fn get_magic_len(&self) -> Result<u16, String> {
        if !self.is_magic {
            return Err(String::from("Not finders fee BlockId"));
        }
        return Ok(self.value as u16 & 0x3fff);
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BID:{}", self.value)
    }
}

impl Serialize for BlockId {
    fn from_serialized(data: &[u8]) -> BlockId {
        BlockId {
            is_continuation: data[0] & 0x80 > 0,
            is_magic: data[0] & 0x40 > 0,
            value: (((data[0] & 0x7) as u16) << 8) + data[1] as u16,
        }
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
        let mut first_byte = (self.value >> 8) as u8;
        if self.is_continuation {
            first_byte ^= 0x80;
        }
        if self.is_magic {
            first_byte ^= 0x40;
        }
        return Ok(vec![first_byte, (self.value & 0xff) as u8]);
    }
}

use crate::user::User;

pub trait Serialize {
    fn from_serialized(data: &[u8], i: &mut usize) -> Result<Box<Self>, String>;
    fn serialize_into(
        &mut self,
        buffer: &mut [u8],
        i: &mut usize,
        users: &mut Vec<User>,
    ) -> Result<usize, String>;
    fn serialized_len(&self) -> Result<usize, String>;
}

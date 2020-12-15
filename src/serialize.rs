pub trait Serialize {
    fn from_serialized(data: &[u8]) -> Result<Box<Self>, String>;
    fn serialize(&mut self) -> Result<Vec<u8>, String>;
    fn serialize_into(&mut self, buffer: &mut [u8]) -> Result<Vec<u8>, String>;
    fn serialized_len(&self) -> Result<usize, String>;
}

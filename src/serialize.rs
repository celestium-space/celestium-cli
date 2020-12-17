pub trait Serialize {
    fn from_serialized(data: &[u8], i: &mut usize) -> Result<Box<Self>, String>;
    //fn my_serialize(&mut self) -> Result<Vec<u8>, String>;
    fn serialize_into(&mut self, buffer: &mut [u8], i: &mut usize) -> Result<usize, String>;
    fn serialized_len(&self) -> Result<usize, String>;
}

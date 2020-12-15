use crate::block::{Block, BlockHash};
use crate::serialize::Serialize;
use openssl::{ec::EcKey, pkey::Public, sha::sha256};

pub struct Blockchain {
    blocks: Vec<Block>,
}

fn get_block_finders_fee(_: usize) -> i32 {
    return 1337;
}

impl Blockchain {
    pub fn new(blocks: Vec<Block>) -> Blockchain {
        Blockchain { blocks: blocks }
    }

    pub fn get_user_value_change(&mut self, pk: &mut EcKey<Public>) -> Result<i32, String> {
        let mut tmp_value = 0;
        for (i, block) in self.blocks.iter_mut().enumerate() {
            tmp_value += block.get_user_value_change(pk)?;
            if pk.public_key_to_der().unwrap() == block.finder.public_key_to_der().unwrap() {
                tmp_value += get_block_finders_fee(i);
            }
        }
        return Ok(tmp_value);
    }
}

impl Serialize for Blockchain {
    fn from_serialized(data: &[u8]) -> Result<Box<Blockchain>, String> {
        let mut i = 0;
        let mut hash = BlockHash::new(0);
        let mut tmp_blocks = Vec::new();
        while i < data.len() {
            let block = *Block::from_serialized(&data[i..])?;
            if block.back_hash == hash {
                let block_len = block.serialized_len()?;
                hash = *BlockHash::from_serialized(&sha256(&data[i..i + block_len]))?;
                let valid_hash = !hash.contains_enough_work();
                if valid_hash {
                    return Err(format!(
                        "Block {} with magic {:x?} does not represent enough work",
                        i, block.magic
                    ));
                }
                tmp_blocks.push(block);
                i += block_len;
            } else {
                return Err(format!(
                    "Block {} in chain has wrong back hash. Expected {} got {}",
                    i, block.back_hash, hash
                ));
            }
        }
        return Ok(Box::new(Blockchain { blocks: tmp_blocks }));
    }
    fn serialize(&mut self) -> Result<Vec<u8>, String> {
        let mut serialized = Vec::new();
        let mut hash = BlockHash::new(0);
        for (i, block) in self.blocks.iter_mut().enumerate() {
            if block.back_hash != hash {
                return Err(format!(
                    "Block {} in chain has wrong back hash. Expected {} got {}",
                    i, hash, block.back_hash
                ));
            }
            let mut serialized_block = block.serialize()?;
            hash = *BlockHash::from_serialized(&sha256(serialized_block.as_slice()))?;
            serialized.append(&mut serialized_block);
        }
        return Ok(serialized);
    }
    fn serialize_into(&mut self, _: &mut [u8]) -> Result<Vec<u8>, String> {
        todo!()
    }

    fn serialized_len(&self) -> Result<usize, String> {
        let mut tmp_len = 0usize;
        for block in &self.blocks {
            tmp_len += block.serialized_len()?;
        }
        return Ok(tmp_len);
    }
}

use crate::block::{Block, BlockHash};
use crate::serialize::Serialize;
use crate::user::User;
use sha2::{Digest, Sha256};

pub struct Blockchain {
    users: Vec<User>,
    blocks: Vec<Block>,
}

// fn get_block_finders_fee(_: usize) -> i32 {
//     return 1337;
// }

impl Blockchain {
    pub fn new(blocks: Vec<Block>) -> Blockchain {
        Blockchain { blocks: blocks }
    }

    // pub fn get_user_value_change(&mut self, pk: &mut PublicKey) -> Result<i32, String> {
    //     let mut tmp_value = 0;
    //     for (i, block) in self.blocks.iter_mut().enumerate() {
    //         tmp_value += block.get_user_value_change(pk)?;
    //         if pk == block.finder {
    //             tmp_value += get_block_finders_fee(i);
    //         }
    //     }
    //     return Ok(tmp_value);
    // }
}

impl Serialize for Blockchain {
    fn from_serialized(data: &[u8], mut i: &mut usize) -> Result<Box<Blockchain>, String> {
        let mut hash = BlockHash::new(0);
        let mut tmp_blocks = Vec::new();
        while *i < data.len() {
            let block = *Block::from_serialized(&data, &mut i)?;
            if block.back_hash == hash {
                let block_len = block.serialized_len()?;
                let mut j = 0;
                hash = *BlockHash::from_serialized(
                    Sha256::digest(&data[*i - (block_len - 1)..*i]).as_slice(),
                    &mut j,
                )?;
                *i += j;
                let valid_hash = hash.contains_enough_work();
                if !valid_hash {
                    return Err(format!(
                        "Block {} with magic {:x?} does not represent enough work",
                        i, block.magic
                    ));
                }
                tmp_blocks.push(block);
            } else {
                return Err(format!(
                    "Block {} in chain has wrong back hash. Expected {} got {}",
                    i, block.back_hash, hash
                ));
            }
        }
        return Ok(Box::new(Blockchain { blocks: tmp_blocks }));
    }
    // fn my_serialize(&mut self) -> Result<Vec<u8>, String> {
    //     let mut serialized = Vec::new();
    //     let mut hash = BlockHash::new(0);
    //     for (i, block) in self.blocks.iter_mut().enumerate() {
    //         if block.back_hash != hash {
    //             return Err(format!(
    //                 "Block {} in chain has wrong back hash. Expected {} got {}",
    //                 i, hash, block.back_hash
    //             ));
    //         }
    //         let mut serialized_block = block.my_serialize()?;
    //         let mut i = 0;
    //         hash = *BlockHash::from_serialized(&Sha256::digest(&serialized_block), &mut i)?;
    //         serialized.append(&mut serialized_block);
    //     }
    //     return Ok(serialized);
    // }
    fn serialize_into(&mut self, data: &mut [u8], mut i: &mut usize) -> Result<usize, String> {
        let mut hash = BlockHash::new(0);
        let orig_i = *i;
        for block in self.blocks.iter_mut() {
            if block.back_hash != hash {
                return Err(format!(
                    "Block {} in chain has wrong back hash. Expected {} got {}",
                    i, hash, block.back_hash
                ));
            }
            let pre_i = *i;
            block.serialize_into(data, &mut i)?;
            let mut j = 0;
            hash = *BlockHash::from_serialized(&Sha256::digest(&data[*i - pre_i..*i]), &mut j)?;
            *i += j;
        }
        return Ok(*i - orig_i);
    }

    fn serialized_len(&self) -> Result<usize, String> {
        let mut tmp_len = 0usize;
        for block in &self.blocks {
            tmp_len += block.serialized_len()?;
        }
        return Ok(tmp_len);
    }
}

use crate::serialize::Serialize;
use crate::transaction::TransactionBlock;
use crate::user_id::UserId;
use secp256k1::PublicKey;
use std::fmt;

pub struct BlockHash {
    value: u32,
}

impl BlockHash {
    pub fn new(value: u32) -> BlockHash {
        BlockHash { value: value }
    }

    pub fn contains_enough_work(&self) -> bool {
        return true;
    }
}

impl PartialEq for BlockHash {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:x}", self.value)
    }
}

impl Serialize for BlockHash {
    fn from_serialized(data: &[u8], i: &mut usize) -> Result<Box<BlockHash>, String> {
        let block_hash = BlockHash {
            value: ((data[*i] as u32) << 24)
                + ((data[*i + 1] as u32) << 16)
                + ((data[*i + 2] as u32) << 8)
                + (data[*i + 3] as u32),
        };
        *i += block_hash.serialized_len()?;
        Ok(Box::new(block_hash))
    }
    // fn my_serialize(&mut self) -> Result<Vec<u8>, String> {
    //     let mut buffer = [0; 4];
    //     buffer[0] = (self.value >> 24) as u8;
    //     buffer[1] = (self.value >> 16) as u8;
    //     buffer[2] = (self.value >> 8) as u8;
    //     buffer[3] = self.value as u8;
    //     return Ok(buffer.to_vec());
    // }
    fn serialize_into(&mut self, buffer: &mut [u8], i: &mut usize) -> Result<usize, String> {
        buffer[*i + 0] = (self.value >> 24) as u8;
        buffer[*i + 1] = (self.value >> 16) as u8;
        buffer[*i + 2] = (self.value >> 8) as u8;
        buffer[*i + 3] = self.value as u8;
        *i += self.serialized_len()?;
        return Ok(self.serialized_len()?);
    }

    fn serialized_len(&self) -> Result<usize, String> {
        return Ok(4);
    }
}

pub struct Block {
    transaction_blocks: Vec<TransactionBlock>,
    uid: UserId,
    pub back_hash: BlockHash,
    pub finder: PublicKey,
    pub magic: Vec<u8>,
}

impl Block {
    pub fn new(
        transactions: Vec<TransactionBlock>,
        uid: UserId,
        back_hash: BlockHash,
        finder: PublicKey,
        magic: Vec<u8>,
    ) -> Block {
        Block {
            transaction_blocks: transactions,
            uid,
            back_hash,
            finder,
            magic,
        }
    }

    pub fn get_user_value_change(&mut self, pk: &mut PublicKey) -> Result<i32, String> {
        let mut tmp_value = 0;
        for transaction_block in self.transaction_blocks.iter_mut() {
            tmp_value += transaction_block.get_user_value_change(pk)?;
        }
        return Ok(tmp_value);
    }
}

impl Serialize for Block {
    fn from_serialized(data: &[u8], mut i: &mut usize) -> Result<Box<Block>, String> {
        let mut transactions = Vec::new();
        let mut uid;
        loop {
            let mut j = *i;
            uid = *UserId::from_serialized(&data, &mut j)?;
            if !uid.is_magic() {
                let transaction = *TransactionBlock::from_serialized(&data, &mut i)?;
                transactions.push(transaction);
            } else {
                break;
            }
        }
        *i += uid.serialized_len()?;
        let magic_len = data[*i];
        let back_hash = *BlockHash::from_serialized(&data, &mut i)?;
        let finder = *PublicKey::from_serialized(&data, &mut i)?;
        let magic = data[*i..*i + magic_len as usize].to_vec();
        *i += magic_len as usize;
        return Ok(Box::new(Block {
            transaction_blocks: transactions,
            back_hash: back_hash,
            finder: finder,
            uid,
            magic: magic,
        }));
    }

    // fn my_serialize(&mut self) -> Result<Vec<u8>, String> {
    //     let mut serialized = Vec::new();
    //     for transaction in self.transaction_blocks.iter_mut() {
    //         serialized.append(&mut transaction.my_serialize()?);
    //     }

    //     serialized.append(
    //         &mut UserId::new(false, true, self.magic.len() as u16)
    //             .my_serialize()
    //             .unwrap(),
    //     );
    //     serialized.append(&mut vec![self.magic.len() as u8]);
    //     serialized.append(&mut self.back_hash.my_serialize().unwrap());
    //     serialized.append(&mut self.finder.serialize().to_vec());
    //     serialized.append(&mut self.magic);
    //     return Ok(serialized);
    // }

    fn serialize_into(&mut self, data: &mut [u8], i: &mut usize) -> Result<usize, String> {
        let start_i = *i;
        for transaction_block in self.transaction_blocks.iter_mut() {
            transaction_block.serialize_into(data, i)?;
        }

        &mut UserId::new(false, true, self.magic.len() as u16).serialize_into(data, i);
        data[*i] = self.magic.len() as u8;
        self.back_hash.serialize_into(data, i)?;
        self.finder.serialize_into(data, i)?;
        for j in 0..self.magic.len() {
            data[*i + j] = self.magic[j];
        }
        return Ok(*i - start_i);
    }

    fn serialized_len(&self) -> Result<usize, String> {
        let mut tmp_len = 0usize;
        for transaction_block in &self.transaction_blocks {
            tmp_len += transaction_block.serialized_len()?;
        }
        let len = tmp_len
            + self.uid.serialized_len()?
            + 1
            + 4
            + self.finder.serialized_len()?
            + self.magic.len();
        return Ok(len);
    }
}

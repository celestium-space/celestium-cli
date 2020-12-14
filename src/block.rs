use crate::block_id::BlockId;
use crate::serialize::Serialize;
use crate::transaction::TransactionBlock;
use openssl::ec::EcKey;
use openssl::pkey::Public;
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
    fn from_serialized(data: &[u8]) -> Result<Box<BlockHash>, String> {
        Ok(Box::new(BlockHash {
            value: ((data[0] as u32) << 24)
                + ((data[1] as u32) << 16)
                + ((data[2] as u32) << 8)
                + (data[3] as u32),
        }))
    }
    fn serialize(&mut self) -> Result<Vec<u8>, String> {
        let mut buffer = [0; 4];
        buffer[0] = (self.value >> 24) as u8;
        buffer[1] = (self.value >> 16) as u8;
        buffer[2] = (self.value >> 8) as u8;
        buffer[3] = self.value as u8;
        return Ok(buffer.to_vec());
    }
    fn serialize_into(&mut self, buffer: &mut [u8]) -> Result<Vec<u8>, String> {
        let serialized = (*self.serialize()?).to_vec();
        buffer[0] = serialized[0];
        buffer[1] = serialized[1];
        buffer[2] = serialized[2];
        buffer[3] = serialized[3];
        return Ok(serialized);
    }

    fn serialized_len(&mut self) -> Result<usize, String> {
        return Ok(4);
    }
}

pub struct Block {
    transaction_blocks: Vec<TransactionBlock>,
    bid: BlockId,
    pub back_hash: BlockHash,
    pub finder: EcKey<Public>,
    pub magic: Vec<u8>,
}

impl Block {
    pub fn new(
        transactions: Vec<TransactionBlock>,
        bid: BlockId,
        back_hash: BlockHash,
        finder: EcKey<Public>,
        magic: Vec<u8>,
    ) -> Block {
        Block {
            transaction_blocks: transactions,
            bid,
            back_hash,
            finder,
            magic,
        }
    }

    pub fn get_user_value_change(&mut self, pk: &mut EcKey<Public>) -> Result<i32, String> {
        let mut tmp_value = 0;
        for transaction_block in self.transaction_blocks.iter_mut() {
            tmp_value += transaction_block.get_user_value_change(pk)?;
        }
        return Ok(tmp_value);
    }
}

impl Serialize for Block {
    fn from_serialized(data: &[u8]) -> Result<Box<Block>, String> {
        let mut transactions = Vec::new();
        let mut i = 0;
        let mut bid;
        loop {
            bid = *BlockId::from_serialized(&data[i..])?;
            if !bid.is_magic() {
                let transaction = *TransactionBlock::from_serialized(&data[i..])?;
                transactions.push(transaction);
                i += transactions.last().unwrap().len();
            } else {
                i += 2;
                break;
            }
        }
        let magic_len = bid.get_magic_len()?;
        return Ok(Box::new(Block {
            transaction_blocks: transactions,
            back_hash: *BlockHash::from_serialized(&data[i..i + 4])?,
            finder: EcKey::public_key_from_der(&data[i + 4..i + 95]).unwrap(),
            bid,
            magic: data[i + 95..i + 95 + magic_len as usize].to_vec(),
        }));
    }

    fn serialize(&mut self) -> Result<Vec<u8>, String> {
        let mut serialized = Vec::new();
        for transaction in self.transaction_blocks.iter_mut() {
            serialized.append(&mut transaction.serialize()?);
        }

        serialized.append(
            &mut BlockId::new(false, true, self.magic.len() as u16)
                .serialize()
                .unwrap(),
        );
        serialized.append(&mut self.back_hash.serialize().unwrap());
        serialized.append(&mut self.finder.public_key_to_der().unwrap());
        serialized.append(&mut self.magic);
        return Ok(serialized);
    }

    fn serialize_into(&mut self, _: &mut [u8]) -> Result<Vec<u8>, String> {
        todo!()
    }

    fn serialized_len(&mut self) -> Result<usize, String> {
        let mut tmp_len = 0usize;
        for transaction_block in self.transaction_blocks.iter_mut() {
            tmp_len += transaction_block.serialized_len()?;
        }
        let len = tmp_len + self.bid.serialized_len()? + 4 + 91 + self.magic.len();
        return Ok(len);
    }
}

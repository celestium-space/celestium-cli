use crate::block_id::BlockId;
use crate::serialize::Serialize;
use crate::transaction::TransactionBlock;
use openssl::ec::EcKey;
use openssl::pkey::Public;

pub struct BlockHash {
    value: u32,
}

impl BlockHash {
    pub fn new(value: u32) -> BlockHash {
        BlockHash { value: value }
    }
}

impl Serialize for BlockHash {
    fn from_serialized(data: &[u8]) -> BlockHash {
        BlockHash {
            value: ((data[0] as u32) << 24)
                + ((data[1] as u32) << 16)
                + ((data[2] as u32) << 8)
                + (data[3] as u32),
        }
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
        match self.serialize() {
            Ok(s) => {
                buffer[0] = s[0];
                buffer[1] = s[1];
                buffer[2] = s[2];
                buffer[3] = s[3];
                return Ok(s);
            }
            Err(e) => Err(e),
        }
    }
}

pub struct Block {
    transactions: Vec<TransactionBlock>,
    back_hash: BlockHash,
    finder: EcKey<Public>,
    magic: Vec<u8>,
}

impl Block {
    pub fn new(
        transactions: Vec<TransactionBlock>,
        back_hash: BlockHash,
        finder: EcKey<Public>,
        magic: Vec<u8>,
    ) -> Block {
        Block {
            transactions: transactions,
            back_hash: back_hash,
            finder: finder,
            magic: magic,
        }
    }
}

impl Serialize for Block {
    fn from_serialized(data: &[u8]) -> Block {
        let transactions = Vec::new();
        let mut i = 0;
        let mut bid = BlockId::from_serialized(&data[i..]);
        loop {
            bid = BlockId::from_serialized(&data[i..]);
            if !bid.is_magic() {
                let transaction: TransactionBlock = TransactionBlock::from_serialized(&data[i..]);
                i += transaction.len();
            } else {
                i += 2;
                break;
            }
        }
        return Block {
            transactions: transactions,
            back_hash: BlockHash::from_serialized(&data[i..i + 4]),
            finder: EcKey::public_key_from_der(&data[i + 4..i + 95]).unwrap(),
            magic: data[i + 95..i + 95 + bid.get_magic_len().unwrap() as usize].to_vec(),
        };
    }

    fn serialize(&mut self) -> Result<Vec<u8>, String> {
        let mut serialized = Vec::new();
        for transaction in self.transactions.iter_mut() {
            match transaction.serialize() {
                Ok(mut s) => serialized.append(&mut s),
                Err(e) => return Err(e.to_string()),
            }
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
        return Err(String::from("Not implemented"));
    }

    // pub fn verify(&self) -> bool {
    //     for transaction in self.transactions {
    //         if !transaction.verify() {
    //             return false;
    //         }
    //     }
    //     return true;
    // }
}

use crate::{
    block::{Block, BlockHash},
    block_id::BlockId,
    blockchain::Blockchain,
    serialize::Serialize,
    transaction::{Transaction, TransactionBlock, TransactionValue},
};
use openssl::{
    ec::EcKey,
    pkey::{Private, Public},
};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

pub struct Wallet {
    blockchain: Blockchain,
    pk: EcKey<Public>,
    sk: EcKey<Private>,
}

impl Wallet {
    pub fn from_binary(
        path: PathBuf,
        pk_path: PathBuf,
        sk_path: PathBuf,
    ) -> Result<Wallet, String> {
        let buffer: &mut Vec<u8> = &mut Vec::new();
        let mut f = File::open(path).unwrap();
        f.read_to_end(buffer).unwrap();
        return Ok(Wallet {
            blockchain: *Blockchain::from_serialized(buffer)?,
            pk: Wallet::load_public_key_from_file(&pk_path),
            sk: Wallet::load_secret_key_from_file(&sk_path),
        });
    }

    pub fn get_balance(&mut self) -> Result<i32, String> {
        return self.blockchain.get_user_value_change(&mut self.pk);
    }

    fn load_public_key_from_file(public_key_file_location: &PathBuf) -> EcKey<Public> {
        let mut f = File::open(public_key_file_location).unwrap();
        let buffer: &mut Vec<u8> = &mut Vec::new();
        f.read_to_end(buffer).unwrap();
        return EcKey::public_key_from_pem(buffer).unwrap();
    }
    fn load_secret_key_from_file(secret_key_file_location: &PathBuf) -> EcKey<Private> {
        let mut f = File::open(secret_key_file_location).unwrap();
        let buffer: &mut Vec<u8> = &mut Vec::new();
        f.read_to_end(buffer).unwrap();
        return EcKey::private_key_from_pem(buffer).unwrap();
    }
    fn create_test_blockchain(location: String) {
        // Create test blockchain
        let pk1_location = PathBuf::from("keys/key1.pub");
        let sk1_location = PathBuf::from("keys/key1");
        let pk2_location = PathBuf::from("keys/key2.pub");
        let sk2_location = PathBuf::from("keys/key2");

        let transaction1 = Transaction::new(
            BlockId::new(true, false, 0x341),
            Wallet::load_public_key_from_file(&pk1_location),
            Wallet::load_public_key_from_file(&pk2_location),
            TransactionValue::new(400, Some(10)),
        );
        let transaction2 = Transaction::new(
            BlockId::new(false, false, 0x341),
            Wallet::load_public_key_from_file(&pk2_location),
            Wallet::load_public_key_from_file(&pk1_location),
            TransactionValue::new(500, Some(25)),
        );
        let mut transaction_block: TransactionBlock =
            TransactionBlock::new(vec![transaction1, transaction2], 2);
        transaction_block.sign(sk1_location);
        transaction_block.sign(sk2_location);
        let block = Block::new(
            vec![transaction_block],
            BlockId::new(false, true, 2),
            BlockHash::new(0),
            Wallet::load_public_key_from_file(&pk1_location),
            vec![0x13, 0x37],
        );
        let mut blockchain = Blockchain::new(vec![block]);

        // Serialize and save blockchain to file
        match blockchain.serialize() {
            Ok(data) => {
                println!(
                    "Block created from parameters and verified, saving to '{}'",
                    location
                );

                let mut f = File::create(location).unwrap();
                f.write_all(data.as_slice()).unwrap();
                drop(f);
            }
            Err(e) => println!("Block creation error: {}", e),
        };
    }
}

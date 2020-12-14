use celestium::block::{Block, BlockHash};
use celestium::block_id::BlockId;
use celestium::serialize::Serialize;
use celestium::transaction::{Transaction, TransactionBlock, TransactionValue};
use openssl::ec::EcKey;
// use openssl::nid::Nid;
use openssl::pkey::Public;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::str;

fn main() {
    // Generate key-pair
    // generate_ecdsa("keys/key2.pub", "keys/key2");

    // Message signing flow check
    // let message = "Hello, world!".as_bytes();
    // let signature = sign(message, "keys/key1").unwrap();
    // println!("Signature: {:02X?}", signature);
    // let matches = verify(message, signature.as_slice(), "keys/key1.pub");
    // match matches {
    //     Ok(v) => {
    //         if v {
    //             println!("Signature matches!")
    //         } else {
    //             println!("Signature doesn't match...")
    //         }
    //     }
    //     Err(e) => println!("Error: {}", e),
    // }
    //let transaction = Transaction();

    // Create transactions and save to file
    let transaction1 = Transaction::new(
        BlockId::new(true, false, 0x341),
        load_public_key_from_file("keys/key1.pub"),
        load_public_key_from_file("keys/key2.pub"),
        TransactionValue::new(-1, None),
    );
    let transaction2 = Transaction::new(
        BlockId::new(false, false, 0x341),
        load_public_key_from_file("keys/key2.pub"),
        load_public_key_from_file("keys/key1.pub"),
        TransactionValue::new(-1, None),
    );
    let mut transaction_block: TransactionBlock =
        TransactionBlock::new(vec![transaction1, transaction2], 2);
    transaction_block.sign("keys/key1");
    transaction_block.sign("keys/key2");
    let mut block = Block::new(
        vec![transaction_block],
        BlockHash::new(0),
        load_public_key_from_file("keys/key1.pub"),
        vec![0x13, 0x37],
    );
    let binary_file_name = "celestium.bin";
    let mut f = File::create(binary_file_name).unwrap();
    match block.serialize() {
        Ok(_) => {
            println!(
                "Block created from parameters and verified, saving to {}",
                binary_file_name
            );

            f.write_all(&block.serialize().unwrap()).unwrap();
            drop(f);

            // Load transactions from file
            let buffer: &mut Vec<u8> = &mut Vec::new();
            let binary_file_name = "celestium.bin";
            let mut f = File::open(binary_file_name).unwrap();
            f.read_to_end(buffer).unwrap();
            let mut block = Block::from_serialized(buffer);

            match block.serialize() {
                Ok(b) => println!("Block loaded from binary and verified! Good job! {:?}", b),
                Err(e) => println!("Block load error: {}", e),
            }
        }
        Err(e) => println!("Block creation error: {}", e),
    }
}

fn load_public_key_from_file(public_key_file_location: &str) -> EcKey<Public> {
    let mut f = File::open(public_key_file_location).unwrap();
    let buffer: &mut Vec<u8> = &mut Vec::new();
    f.read_to_end(buffer).unwrap();
    let public_key = EcKey::public_key_from_pem(buffer).unwrap();
    return public_key;
}

// fn generate_ec_keys(pk_file_location: &str, sk_file_location: &str) {
//     let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
//     let key = EcKey::generate(&group).unwrap();

//     let mut pk_file = File::create(pk_file_location).unwrap();
//     let mut sk_file = File::create(sk_file_location).unwrap();
//     pk_file
//         .write_all(key.public_key_to_pem().unwrap().as_slice())
//         .unwrap();
//     sk_file
//         .write_all(key.private_key_to_pem().unwrap().as_slice())
//         .unwrap();
//     return;
// }

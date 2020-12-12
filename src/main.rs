use celestium::transaction::{TransactionBlock, TransactionBlockTrait};
use openssl::ec::EcKey;
// use openssl::nid::Nid;
use openssl::pkey::Public;
use std::fs::File;
// use std::io::prelude::*;
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

    let pk1_1 = load_public_key_from_file("keys/key1.pub");
    let pk2_1 = load_public_key_from_file("keys/key2.pub");
    //let pk1_2 = load_public_key_from_file("keys/key1.pub");
    //let pk2_2 = load_public_key_from_file("keys/key2.pub");
    let value = 42;
    let mut buffer1 = tmp_serialize(1, pk1_1, value, pk2_1);
    //let mut buffer2 = tmp_serialize(2, pk2_2, value, pk1_2);
    let mut buffer = Vec::new();
    buffer.append(&mut buffer1);
    //buffer.append(&mut buffer2);
    let mut trans: TransactionBlock = TransactionBlockTrait::new(buffer.to_vec());
    trans.sign("keys/key1");
    //trans.sign("keys/key2");
    println!("{:?}", trans.verify());
}

fn tmp_serialize(bid: i16, from_pk: EcKey<Public>, value: u32, to_pk: EcKey<Public>) -> Vec<u8> {
    let mut buffer: [u8; 189] = [0; 189];
    buffer[0] = (bid >> 8) as u8;
    buffer[1] = bid as u8;
    buffer[2..93].clone_from_slice(&mut from_pk.public_key_to_der().unwrap());
    buffer[94] = (value >> 24) as u8;
    buffer[95] = (value >> 16) as u8;
    buffer[96] = (value >> 8) as u8;
    buffer[97] = value as u8;
    buffer[98..189].clone_from_slice(&mut to_pk.public_key_to_der().unwrap());
    return buffer.to_vec();
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

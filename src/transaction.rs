use openssl::ec::EcKey;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Public};
use openssl::sign::{Signer, Verifier};
use std::fs::File;
use std::io::prelude::*;

pub struct Transaction {
    bid: i16,
    to_pk: EcKey<Public>,
    value: u32,
    from_pk: EcKey<Public>,
}

pub struct TransactionBlock {
    transactions: Vec<Transaction>,
    pub signatures: Vec<Vec<u8>>,
}

pub trait TransactionBlockTrait {
    fn new(data: Vec<u8>) -> Self;
    fn serialize(&self) -> Vec<u8>;
    fn sign(&mut self, pk_file_location: &str);
    fn verify(&self) -> bool;
}

impl TransactionBlockTrait for TransactionBlock {
    fn new(data: Vec<u8>) -> TransactionBlock {
        let mut i = 0;
        let mut transactions = Vec::new();
        loop {
            let bid = ((data[i] as i16) << 8) + data[i + 1] as i16;
            transactions.push(Transaction {
                bid: bid,
                from_pk: EcKey::public_key_from_der(&data[i + 2..i + 93]).unwrap(),
                value: ((data[i + 94] as u32) << 24)
                    + ((data[i + 95] as u32) << 16)
                    + ((data[i + 96] as u32) << 8)
                    + (data[i + 97] as u32),
                to_pk: EcKey::public_key_from_der(&data[i + 98..i + 189]).unwrap(),
            });
            i += 189;
            if bid > 0 {
                break;
            }
        }
        // let signatures = vec![];
        // for j in 0..transactions.len() {
        //     let data_start = i * 189 + j * ;
        //     signatures.push()
        // }
        TransactionBlock {
            transactions: transactions,
            signatures: Vec::new(),
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut return_buffer: Vec<u8> = Vec::new();
        for transaction in &self.transactions {
            let mut buffer: [u8; 189] = [0; 189];
            buffer[0] = (transaction.bid >> 8) as u8;
            buffer[1] = transaction.bid as u8;
            buffer[2..93].clone_from_slice(&mut transaction.from_pk.public_key_to_der().unwrap());
            buffer[94] = (transaction.value >> 24) as u8;
            buffer[95] = (transaction.value >> 16) as u8;
            buffer[96] = (transaction.value >> 8) as u8;
            buffer[97] = transaction.value as u8;
            buffer[98..189].clone_from_slice(&mut transaction.to_pk.public_key_to_der().unwrap());
            return_buffer.append(&mut buffer.to_vec());
        }
        return return_buffer;
    }

    fn sign(&mut self, sk_file_location: &str) {
        let mut f = File::open(sk_file_location).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        let ec_key = EcKey::private_key_from_pem(buffer.as_slice()).unwrap();
        let p_key = PKey::from_ec_key(ec_key).unwrap();
        let mut signer = Signer::new(MessageDigest::sha256(), &p_key).unwrap();
        let bytes = &self.serialize();
        signer.update(bytes).unwrap();
        self.signatures.push(signer.sign_to_vec().unwrap());
    }
    fn verify(&self) -> bool {
        let mut seen_pks: Vec<Vec<u8>> = Vec::new();
        let mut i = 0;
        for transaction in &self.transactions {
            if seen_pks.contains(&transaction.from_pk.public_key_to_der().unwrap()) {
                continue;
            }
            if i > self.signatures.len() - 1 {
                return false;
            }
            let signature = &self.signatures[i];
            let ecder_vec = transaction.from_pk.public_key_to_der().unwrap();
            let ecder_slice = ecder_vec.as_slice();
            let p_key = PKey::public_key_from_der(ecder_slice).unwrap();
            let mut verifier = Verifier::new(MessageDigest::sha256(), &p_key).unwrap();
            let bytes = &self.serialize();
            if !verifier
                .verify_oneshot(signature.as_slice(), bytes)
                .unwrap()
            {
                return false;
            }
            seen_pks.push(transaction.from_pk.public_key_to_der().unwrap());
            i += 1;
        }
        return true;
    }
}

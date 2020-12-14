use crate::block_id::BlockId;
use crate::serialize::Serialize;
use openssl::ec::EcKey;
use openssl::ecdsa::EcdsaSig;
use openssl::pkey::Public;
use std::fs::File;
use std::io::prelude::*;

pub struct TransactionValue {
    value: i32,
    fee: Option<u8>,
}

impl TransactionValue {
    pub fn new(value: i32, fee: Option<u8>) -> TransactionValue {
        TransactionValue {
            value: value,
            fee: fee,
        }
    }
}

impl Serialize for TransactionValue {
    fn from_serialized(data: &[u8]) -> TransactionValue {
        let mut tmp_val = ((data[0] as i32) << 24)
            + ((data[1] as i32) << 16)
            + ((data[2] as i32) << 8)
            + (data[3] as i32);
        let mut tmp_fee = None;
        if tmp_val >= 0 {
            tmp_fee = Some((tmp_val & 0xff) as u8);
            tmp_val = tmp_val >> 8;
        }
        TransactionValue {
            value: tmp_val,
            fee: tmp_fee,
        }
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

    fn serialize(&mut self) -> Result<Vec<u8>, String> {
        let mut buffer = [0; 4];
        let mut tmp_val = self.value;
        if tmp_val >= 0 {
            match self.fee {
                Some(f) => tmp_val = (tmp_val << 8) + (f as i32),
                None => {
                    return Err(String::from(
                        "Transaction error: Undefined fee for coin transfer",
                    ))
                }
            }
        }
        buffer[0] = (tmp_val >> 24) as u8;
        buffer[1] = (tmp_val >> 16) as u8;
        buffer[2] = (tmp_val >> 8) as u8;
        buffer[3] = tmp_val as u8;
        return Ok(buffer[0..4].to_vec());
    }
}

pub struct Transaction {
    bid: BlockId,
    from_pk: EcKey<Public>,
    to_pk: EcKey<Public>,
    value: TransactionValue,
}

impl Transaction {
    pub fn new(
        bid: BlockId,
        from_pk: EcKey<Public>,
        to_pk: EcKey<Public>,
        value: TransactionValue,
    ) -> Transaction {
        Transaction {
            bid: bid,
            from_pk: from_pk,
            to_pk: to_pk,
            value: value,
        }
    }
}

pub struct TransactionBlock {
    pub transactions: Vec<Transaction>,
    pub expected_signatures: usize,
    pub signatures: Vec<Vec<u8>>,
}

impl TransactionBlock {
    pub fn new(transactions: Vec<Transaction>, expected_signatures: usize) -> TransactionBlock {
        TransactionBlock {
            transactions: transactions,
            expected_signatures: expected_signatures,
            signatures: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        return self.transactions.len() * 188 + self.signatures.len() * 72;
    }

    pub fn sign(&mut self, sk_file_location: &str) {
        let mut f = File::open(sk_file_location).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        let ec_key = EcKey::private_key_from_pem(buffer.as_slice()).unwrap();
        let bytes = &self.serialize_content().unwrap();
        self.signatures.push(
            EcdsaSig::sign(bytes, ec_key.as_ref())
                .unwrap()
                .to_der()
                .unwrap()
                .to_vec(),
        );
    }

    fn serialize_content(&mut self) -> Result<Vec<u8>, String> {
        let mut return_buffer: Vec<u8> = Vec::new();
        for transaction in self.transactions.iter_mut() {
            let mut buffer: [u8; 188] = [0; 188];
            transaction.bid.serialize_into(&mut buffer)?;
            buffer[2..93].clone_from_slice(&mut transaction.from_pk.public_key_to_der().unwrap());
            buffer[93..184].clone_from_slice(&mut transaction.to_pk.public_key_to_der().unwrap());
            transaction.value.serialize_into(&mut buffer[184..188])?;
            return_buffer.append(&mut buffer.to_vec());
        }
        return Ok(return_buffer);
    }
}

impl Serialize for TransactionBlock {
    fn from_serialized(data: &[u8]) -> TransactionBlock {
        let mut i = 0;
        let mut transactions = Vec::new();
        let mut seen_pks: Vec<Vec<u8>> = Vec::new();
        loop {
            let transaction = Transaction::new(
                BlockId::from_serialized(&data[i..i + 2]),
                EcKey::public_key_from_der(&data[i + 2..i + 93]).unwrap(),
                EcKey::public_key_from_der(&data[i + 93..i + 184]).unwrap(),
                TransactionValue::from_serialized(&data[i + 184..i + 188]),
            );

            transactions.push(transaction);
            if !seen_pks.contains(&data[i + 93..i + 184].to_vec()) {
                seen_pks.push(data[i + 93..i + 184].to_vec());
            }
            i += 188;
            if !transactions.last().unwrap().bid.is_continuation() {
                break;
            }
        }
        let mut tmp_signatures: Vec<Vec<u8>> = Vec::new();
        for _ in 0..seen_pks.len() {
            tmp_signatures.push(data[i..i + 72].to_vec());
            i += 72;
        }
        TransactionBlock {
            transactions: transactions,
            expected_signatures: seen_pks.len(),
            signatures: tmp_signatures,
        }
    }

    fn serialize(&mut self) -> Result<Vec<u8>, String> {
        if self.expected_signatures != self.signatures.len() {
            return Err(format!(
                "Wrong amount of signatures; expected {} got {}",
                self.expected_signatures,
                self.signatures.len()
            ));
        }
        match self.serialize_content() {
            Ok(mut serialized) => {
                let mut seen_pks: Vec<Vec<u8>> = Vec::new();
                for transaction in &self.transactions {
                    if seen_pks.contains(&transaction.from_pk.public_key_to_der().unwrap()) {
                        continue;
                    }
                    if seen_pks.len() > self.signatures.len() {
                        return Err(format!(
                            "Too few signatures, missing signature for transaction {} at least",
                            transaction.bid,
                        ));
                    }
                    let signature = &self.signatures[seen_pks.len()];
                    let bytes = serialized.as_slice();
                    let verifier = EcdsaSig::from_der(signature).unwrap();
                    if !verifier
                        .verify(bytes, transaction.from_pk.as_ref())
                        .unwrap()
                    {
                        return Err(format!("Signature not valid for {}", transaction.bid));
                    }
                    seen_pks.push(transaction.from_pk.public_key_to_der().unwrap());
                }
                for (i, signature) in self.signatures.iter().enumerate() {
                    if signature.len() > 72 {
                        return Err(format!("Signature {} too long", i));
                    }
                    for j in 0..72 {
                        if j < signature.len() {
                            serialized.push(signature[j]);
                        } else {
                            serialized.push(0);
                        }
                    }
                }
                return Ok(serialized);
            }
            Err(e) => Err(e),
        }
    }

    fn serialize_into(&mut self, buffer: &mut [u8]) -> Result<Vec<u8>, String> {
        match self.serialize() {
            Ok(s) => {
                for i in 0..s.len() {
                    buffer[i] = s[i];
                }
                return Ok(s);
            }
            Err(e) => Err(e),
        }
    }
}

use crate::serialize::Serialize;
use crate::user_id::UserId;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey, Signature};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::{io::prelude::*, path::PathBuf};

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

    fn is_coin_transfer(&self) -> Result<bool, String> {
        if self.value >= 0 {
            match self.fee {
                Some(_) => Ok(true),
                None => Err(String::from("Undefined fee for coin transfer")),
            }
        } else {
            match self.fee {
                Some(_) => Err(String::from("Fee on ID transfer")),
                None => Ok(false),
            }
        }
    }

    pub fn get_value(&self) -> Result<u32, String> {
        if self.is_coin_transfer()? {
            return Ok(self.value as u32 - self.fee.unwrap() as u32);
        };
        Err(String::from(
            "Can not get transaction value: Transaction not coin transfer",
        ))
    }
    pub fn get_fee(&self) -> Result<u32, String> {
        if self.is_coin_transfer()? {
            return Ok(self.fee.unwrap() as u32);
        };
        Err(String::from(
            "Can not get transaction fee: Transaction not coin transfer",
        ))
    }
    pub fn get_id(self) -> Result<u32, String> {
        if !self.is_coin_transfer()? {
            return Ok((self.value * -1) as u32);
        };
        Err(String::from("Transaction not ID transfer"))
    }
}

impl Serialize for TransactionValue {
    fn from_serialized(data: &[u8], i: &mut usize) -> Result<Box<TransactionValue>, String> {
        let mut tmp_val = ((data[*i] as i32) << 24)
            + ((data[*i + 1] as i32) << 16)
            + ((data[*i + 2] as i32) << 8)
            + (data[*i + 3] as i32);
        let mut tmp_fee = None;
        if tmp_val >= 0 {
            tmp_fee = Some((tmp_val & 0xff) as u8);
            tmp_val = tmp_val >> 8;
        }
        let transaction_value = TransactionValue {
            value: tmp_val,
            fee: tmp_fee,
        };
        *i += transaction_value.serialized_len()?;
        Ok(Box::new(transaction_value))
    }
    fn serialize_into(&mut self, buffer: &mut [u8], i: &mut usize) -> Result<usize, String> {
        let mut tmp_val = self.value;
        if tmp_val >= 0 {
            tmp_val = (tmp_val << 8) + (self.get_fee()? as i32);
        }
        buffer[*i] = (tmp_val >> 24) as u8;
        buffer[*i + 1] = (tmp_val >> 16) as u8;
        buffer[*i + 2] = (tmp_val >> 8) as u8;
        buffer[*i + 3] = tmp_val as u8;
        *i += 4;
        return Ok(4);
    }

    // fn my_serialize(&mut self) -> Result<Vec<u8>, String> {
    //     let mut buffer = [0; 4];
    //     let mut tmp_val = self.value;
    //     if tmp_val >= 0 {
    //         tmp_val = (tmp_val << 8) + (self.get_fee()? as i32);
    //     }
    //     buffer[0] = (tmp_val >> 24) as u8;
    //     buffer[1] = (tmp_val >> 16) as u8;
    //     buffer[2] = (tmp_val >> 8) as u8;
    //     buffer[3] = tmp_val as u8;
    //     return Ok(buffer[0..4].to_vec());
    // }

    fn serialized_len(&self) -> Result<usize, String> {
        Ok(4)
    }
}

impl Serialize for PublicKey {
    fn from_serialized(data: &[u8], i: &mut usize) -> Result<Box<PublicKey>, String> {
        match PublicKey::from_slice(&data[*i..*i + 33]) {
            Ok(public_key) => {
                *i += public_key.serialized_len()?;
                return Ok(Box::new(public_key));
            }
            Err(e) => Err(format!(
                "Could not deserialize public key {:x?}: {}",
                &data[*i..*i + 33],
                e.to_string()
            )),
        }
    }

    fn serialize_into(&mut self, buffer: &mut [u8], i: &mut usize) -> Result<usize, String> {
        let self_bytes = self.serialize();
        buffer[*i..*i + self_bytes.len()].copy_from_slice(&self_bytes);
        *i += self_bytes.len();
        return Ok(self_bytes.len());
    }
    fn serialized_len(&self) -> Result<usize, String> {
        Ok(self.serialize().len())
    }
}

impl Serialize for SecretKey {
    fn from_serialized(secret_key: &[u8], i: &mut usize) -> Result<Box<SecretKey>, String> {
        match SecretKey::from_slice(secret_key) {
            Ok(secret_key) => {
                *i += secret_key.len();
                return Ok(Box::new(secret_key));
            }
            Err(e) => Err(format!(
                "Could not deserialize secret key {:?}: {}",
                secret_key,
                e.to_string()
            )),
        }
    }

    fn serialize_into(&mut self, buffer: &mut [u8], i: &mut usize) -> Result<usize, String> {
        let self_bytes = self.as_ref();
        buffer.copy_from_slice(self_bytes);
        *i += self_bytes.len();
        return Ok(self_bytes.len());
    }
    // fn my_serialize(&mut self) -> Result<Vec<u8>, String> {
    //     Ok(self.as_ref().to_vec())
    // }
    fn serialized_len(&self) -> Result<usize, String> {
        Ok(self.as_ref().len())
    }
}
pub struct Transaction {
    uid: UserId,
    from_pk: PublicKey,
    to_pk: PublicKey,
    value: TransactionValue,
}

impl Transaction {
    pub fn new(
        uid: UserId,
        from_pk: PublicKey,
        to_pk: PublicKey,
        value: TransactionValue,
    ) -> Transaction {
        Transaction {
            uid: uid,
            from_pk: from_pk,
            to_pk: to_pk,
            value: value,
        }
    }
}

impl Serialize for Transaction {
    fn from_serialized(data: &[u8], mut i: &mut usize) -> Result<Box<Self>, String> {
        let user_id = *UserId::from_serialized(&data, &mut i)?;
        let from_pk = *PublicKey::from_serialized(&data, i).unwrap();
        let to_pk = *PublicKey::from_serialized(&data, i).unwrap();
        let value = *TransactionValue::from_serialized(&data, i)?;
        return Ok(Box::new(Transaction {
            uid: user_id,
            from_pk: from_pk,
            to_pk: to_pk,
            value: value,
        }));
    }

    // fn my_serialize(&mut self) -> Result<Vec<u8>, String> {
    //     //todo!();
    //     let mut return_buffer = Vec::new();
    //     return_buffer.append(&mut self.uid.my_serialize()?);
    //     let mut from_pk_data = self.from_pk.serialize().to_vec();
    //     return_buffer.append(&mut from_pk_data);
    //     return_buffer.append(&mut self.to_pk.serialize().to_vec());
    //     return_buffer.append(&mut self.value.my_serialize()?);
    //     return Ok(return_buffer);
    // }

    fn serialize_into(
        &mut self,
        mut buffer: &mut [u8],
        mut i: &mut usize,
    ) -> Result<usize, String> {
        let start_i = *i;
        self.uid.serialize_into(&mut buffer, i)?;
        self.from_pk.serialize_into(&mut buffer, &mut i)?;
        self.to_pk.serialize_into(&mut buffer, &mut i)?;
        self.value.serialize_into(&mut buffer, &mut i)?;
        return Ok(*i - start_i);
    }

    fn serialized_len(&self) -> Result<usize, String> {
        let transaction_len = self.uid.serialized_len()?
            + self.from_pk.serialized_len()?
            + self.to_pk.serialized_len()?
            + self.value.serialized_len()?;
        Ok(transaction_len)
    }
}

pub struct TransactionBlock {
    pub transactions: Vec<Transaction>,
    pub expected_signatures: usize,
    pub signatures: Vec<Signature>,
}

impl TransactionBlock {
    pub fn new(transactions: Vec<Transaction>, expected_signatures: usize) -> TransactionBlock {
        TransactionBlock {
            transactions: transactions,
            expected_signatures: expected_signatures,
            signatures: Vec::new(),
        }
    }

    pub fn get_user_value_change(&mut self, pk: &mut PublicKey) -> Result<i32, String> {
        let mut tmp_value = 0;
        for transaction in self.transactions.iter_mut() {
            if transaction.value.is_coin_transfer()? {
                let transaction_value = transaction.value.get_value()? as i32;
                if pk == &mut transaction.from_pk {
                    tmp_value -= transaction_value;
                }
                if pk == &mut transaction.to_pk {
                    tmp_value += transaction_value;
                }
            }
        }
        return Ok(tmp_value);
    }

    pub fn len(&self) -> usize {
        return self.transactions.len() * 188 + self.signatures.len() * 72;
    }

    pub fn sign(&mut self, sk_file_location: PathBuf) {
        let mut f = File::open(sk_file_location).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        let secp = Secp256k1::new();
        let mut i = 0;
        let sk = *SecretKey::from_serialized(buffer.as_slice(), &mut i).unwrap();
        let bytes = &self.serialize_content().unwrap();
        let message = Message::from_slice(Sha256::digest(bytes).as_slice()).unwrap();
        self.signatures.push(secp.sign(&message, &sk));
    }

    fn serialize_content(&mut self) -> Result<Vec<u8>, String> {
        if self.transactions.len() > 0 {
            let mut return_buffer =
                vec![0; self.transactions.len() * self.transactions[0].serialized_len()?];
            let mut i = 0;
            for transaction in self.transactions.iter_mut() {
                transaction.serialize_into(&mut return_buffer, &mut i)?;
            }
            return Ok(return_buffer);
        }
        return Ok(Vec::new());
    }
}

impl Serialize for TransactionBlock {
    fn from_serialized(data: &[u8], i: &mut usize) -> Result<Box<TransactionBlock>, String> {
        let mut transactions = Vec::new();
        let mut seen_pks = Vec::new();
        loop {
            let transaction = *Transaction::from_serialized(&data, i)?;
            let from_pk = transaction.from_pk;
            transactions.push(transaction);
            if !seen_pks.contains(&from_pk) {
                seen_pks.push(from_pk);
            }
            if !transactions.last().unwrap().uid.is_continuation() {
                break;
            }
        }
        let mut tmp_signatures: Vec<Signature> = Vec::new();
        for _ in seen_pks.iter() {
            match Signature::from_compact(&data[*i..*i + 64].to_vec()) {
                Ok(signature) => {
                    tmp_signatures.push(signature);
                    *i += signature.serialize_compact().len();
                }
                Err(e) => {
                    return Err(format!(
                        "Could not deserialize signatrue: {}",
                        e.to_string()
                    ))
                }
            }
        }
        Ok(Box::new(TransactionBlock {
            transactions: transactions,
            expected_signatures: seen_pks.len(),
            signatures: tmp_signatures,
        }))
    }

    // fn my_serialize(&mut self) -> Result<Vec<u8>, String> {
    //     if self.expected_signatures != self.signatures.len() {
    //         return Err(format!(
    //             "Wrong amount of signatures; expected {} got {}",
    //             self.expected_signatures,
    //             self.signatures.len()
    //         ));
    //     }
    //     match self.serialize_content() {
    //         Ok(mut serialized) => {
    //             let mut seen_pks: Vec<PublicKey> = Vec::new();
    //             for transaction in &self.transactions {
    //                 if seen_pks.contains(&transaction.from_pk) {
    //                     continue;
    //                 }
    //                 if seen_pks.len() > self.signatures.len() {
    //                     return Err(format!(
    //                         "Too few signatures, missing signature for transaction {} at least",
    //                         transaction.uid,
    //                     ));
    //                 }
    //                 let bytes = serialized.as_slice();
    //                 match Message::from_slice(Sha256::digest(bytes).as_slice()) {
    //                     Ok(message) => {
    //                         let signature = self.signatures[seen_pks.len()];
    //                         let secp = Secp256k1::new();
    //                         if !secp
    //                             .verify(&message, &signature, &transaction.from_pk)
    //                             .is_ok()
    //                         {
    //                             return Err(format!("Signature not valid for {}", transaction.uid));
    //                         }
    //                         seen_pks.push(transaction.from_pk);
    //                     }
    //                     Err(e) => {
    //                         return Err(format!("Could not generate message from bytes: {}", e))
    //                     }
    //                 }
    //             }
    //             for signature in self.signatures.iter() {
    //                 let vec_sig = signature.serialize_compact();
    //                 serialized.append(&mut vec_sig.to_vec());
    //             }
    //             return Ok(serialized);
    //         }
    //         Err(e) => Err(e),
    //     }
    // }

    fn serialize_into(&mut self, buffer: &mut [u8], i: &mut usize) -> Result<usize, String> {
        if self.expected_signatures != self.signatures.len() {
            return Err(format!(
                "Wrong amount of signatures; expected {} got {}",
                self.expected_signatures,
                self.signatures.len()
            ));
        }
        let content_start = *i;
        let mut seen_pks: Vec<PublicKey> = Vec::new();
        for transaction in self.transactions.iter_mut() {
            transaction.serialize_into(buffer, i)?;
            if !seen_pks.contains(&transaction.from_pk) {
                seen_pks.push(transaction.from_pk);
            }
        }
        let content_end = *i;
        if seen_pks.len() != self.signatures.len() {
            return Err(format!(
                "Wrong amount of signatures on transaction, expected {} got {}",
                seen_pks.len(),
                self.signatures.len()
            ));
        }
        match Message::from_slice(Sha256::digest(&buffer[content_start..content_end]).as_slice()) {
            Ok(message) => {
                for (j, signature) in self.signatures.iter().enumerate() {
                    let secp = Secp256k1::new();
                    if secp
                        .verify(&message, &signature, &self.transactions[j].from_pk)
                        .is_err()
                    {
                        return Err(format!(
                            "Signature not valid for {}",
                            self.transactions[j].uid
                        ));
                    }
                    let vec_sig = signature.serialize_compact();
                    buffer[*i..*i + vec_sig.len()].copy_from_slice(&vec_sig);
                    *i += vec_sig.len();
                }
            }
            Err(e) => return Err(format!("Could not generate message from bytes: {}", e)),
        }
        return Ok(*i - content_start);
    }

    fn serialized_len(&self) -> Result<usize, String> {
        let mut tmp_len = 0;
        for transaction in &self.transactions {
            tmp_len += transaction.serialized_len()?;
        }
        for signature in self.signatures.iter() {
            tmp_len += signature.serialize_compact().len();
        }
        return Ok(tmp_len);
    }
}

use crate::{blockchain::Blockchain, serialize::Serialize};
use secp256k1::{PublicKey, SecretKey};
use std::{fs::File, io::Read, path::PathBuf};

pub struct Wallet {
    pub blockchain: Blockchain,
    pub pk: Option<PublicKey>,
    pub sk: Option<SecretKey>,
}

impl Wallet {
    pub fn from_binary(
        path: PathBuf,
        pk_path: Option<PathBuf>,
        sk_path: Option<PathBuf>,
    ) -> Result<Wallet, String> {
        let buffer: &mut Vec<u8> = &mut Vec::new();
        let mut f = File::open(path).unwrap();
        let mut tmp_pk = None;
        let mut tmp_sk = None;
        if pk_path.is_some() && sk_path.is_some() {
            tmp_pk = Some(Wallet::load_public_key_from_file(&pk_path.unwrap()).unwrap());
            tmp_sk = Some(Wallet::load_secret_key_from_file(&sk_path.unwrap()).unwrap());
        }
        f.read_to_end(buffer).unwrap();
        let mut i = 0;
        return Ok(Wallet {
            blockchain: *Blockchain::from_serialized(buffer, &mut i)?,
            pk: tmp_pk,
            sk: tmp_sk,
        });
    }

    // pub fn get_balance(&mut self) -> Result<i32, String> {
    //     match &mut self.pk {
    //         Some(pk) => self.blockchain.get_user_value_change(pk),
    //         None => Err(String::from("Personal keyset not defined")),
    //     }
    // }

    pub fn load_public_key_from_file(
        public_key_file_location: &PathBuf,
    ) -> Result<PublicKey, String> {
        let mut f = File::open(public_key_file_location).unwrap();
        let buffer = &mut Vec::new();
        f.read_to_end(buffer).unwrap();
        let mut i = 0;
        return Ok(*PublicKey::from_serialized(buffer, &mut i)?);
    }
    pub fn load_secret_key_from_file(
        secret_key_file_location: &PathBuf,
    ) -> Result<SecretKey, String> {
        let mut f = File::open(secret_key_file_location).unwrap();
        let data = &mut Vec::new();
        f.read_to_end(data).unwrap();
        let mut i = 0;
        return Ok(*SecretKey::from_serialized(data, &mut i)?);
    }
}

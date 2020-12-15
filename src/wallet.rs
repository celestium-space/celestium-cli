use crate::{blockchain::Blockchain, serialize::Serialize};
use openssl::{
    ec::EcKey,
    pkey::{Private, Public},
};
use std::{fs::File, io::Read, path::PathBuf};

pub struct Wallet {
    blockchain: Blockchain,
    pk: Option<EcKey<Public>>,
    sk: Option<EcKey<Private>>,
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
            tmp_pk = Some(Wallet::load_public_key_from_file(&pk_path.unwrap()));
            tmp_sk = Some(Wallet::load_secret_key_from_file(&sk_path.unwrap()));
        }
        f.read_to_end(buffer).unwrap();
        return Ok(Wallet {
            blockchain: *Blockchain::from_serialized(buffer)?,
            pk: tmp_pk,
            sk: tmp_sk,
        });
    }

    pub fn get_balance(&mut self) -> Result<i32, String> {
        match &mut self.pk {
            Some(pk) => self.blockchain.get_user_value_change(pk),
            None => Err(String::from("Personal keyset not defined")),
        }
    }

    pub fn load_public_key_from_file(public_key_file_location: &PathBuf) -> EcKey<Public> {
        let mut f = File::open(public_key_file_location).unwrap();
        let buffer: &mut Vec<u8> = &mut Vec::new();
        f.read_to_end(buffer).unwrap();
        return EcKey::public_key_from_pem(buffer).unwrap();
    }
    pub fn load_secret_key_from_file(secret_key_file_location: &PathBuf) -> EcKey<Private> {
        let mut f = File::open(secret_key_file_location).unwrap();
        let buffer: &mut Vec<u8> = &mut Vec::new();
        f.read_to_end(buffer).unwrap();
        return EcKey::private_key_from_pem(buffer).unwrap();
    }
}

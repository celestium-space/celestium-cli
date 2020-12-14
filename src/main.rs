use celestium::wallet::Wallet;
use openssl::ec::{EcGroup, EcKey};
use openssl::nid::Nid;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self};
use std::path::Path;
use std::str;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    pattern: String,

    #[structopt(parse(from_os_str))]
    blockchain_path: std::path::PathBuf,

    #[structopt(parse(from_os_str))]
    pk_path: std::path::PathBuf,

    #[structopt(parse(from_os_str))]
    sk_path: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();
    match Wallet::from_binary(args.blockchain_path, args.pk_path, args.sk_path) {
        Ok(mut w) => loop {
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    if input == String::from("balance\n") {
                        println!("User balance: {}", w.get_balance().unwrap());
                    } else if input == String::from("exit\n") {
                        break;
                    } else {
                        println!("Unknown command: '{}'", input);
                    }
                }
                Err(e) => println!("{}", e),
            }
        },
        Err(e) => println!("Could not load blockchain: {}", e),
    }
}
fn generate_key_pair() {
    // Generate key-pair
    let pk1_location = "keys/key1.pub";
    let sk1_location = "keys/key1";
    let pk2_location = "keys/key2.pub";
    let sk2_location = "keys/key2";

    if !Path::new(pk1_location).exists() || !Path::new(sk1_location).exists() {
        generate_ec_keys(pk1_location, sk1_location);
    }
    if !Path::new(pk2_location).exists() || !Path::new(sk2_location).exists() {
        generate_ec_keys(pk2_location, sk2_location);
    }
}

fn generate_ec_keys(pk_file_location: &str, sk_file_location: &str) {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let key = EcKey::generate(&group).unwrap();
    let mut pk_file = File::create(pk_file_location).unwrap();
    let mut sk_file = File::create(sk_file_location).unwrap();
    pk_file
        .write_all(key.public_key_to_pem().unwrap().as_slice())
        .unwrap();
    sk_file
        .write_all(key.private_key_to_pem().unwrap().as_slice())
        .unwrap();
    return;
}

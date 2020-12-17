use celestium::{
    block::{Block, BlockHash},
    blockchain::Blockchain,
    serialize::Serialize,
    transaction::{Transaction, TransactionBlock, TransactionValue},
    universal_id::UniversalId,
    user::User,
    wallet::Wallet,
};
use rand::rngs::OsRng;
use secp256k1::Secp256k1;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, prelude::*, Write},
    path::PathBuf,
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    blockchain_path: Option<std::path::PathBuf>,

    #[structopt(parse(from_os_str))]
    pk_path: Option<std::path::PathBuf>,

    #[structopt(parse(from_os_str))]
    sk_path: Option<std::path::PathBuf>,
}

fn main() {
    let args = Cli::from_args();
    match (args.blockchain_path, args.pk_path, args.sk_path) {
        (Some(bc_path), Some(pk_path), Some(sk_path)) => {
            if bc_path.exists() && pk_path.exists() && sk_path.exists() {
                let mut file = File::open(bc_path).unwrap();
                let mut bp_bin = vec![];
                file.read_to_end(&mut bp_bin).unwrap();
                file = File::open(pk_path).unwrap();
                let mut pk_bin = vec![];
                file.read_to_end(&mut pk_bin).unwrap();
                file = File::open(sk_path).unwrap();
                let mut sk_bin = vec![];
                file.read_to_end(&mut sk_bin).unwrap();
                match Wallet::from_binary(bp_bin, pk_bin, sk_bin) {
                    Ok(mut w) => loop {
                        let mut input = String::new();
                        match io::stdin().read_line(&mut input) {
                            Ok(_) => {
                                let command = String::from(input.trim());
                                if command == String::from("balance") {
                                    match w.get_balance() {
                                        Ok(b) => println!("User balance: {}", b),
                                        Err(e) => println!("Error while getting balance: {}", e),
                                    }
                                } else if command == String::from("exit") {
                                    break;
                                } else {
                                    println!("Unknown command: '{}'", command);
                                }
                            }
                            Err(e) => println!("{}", e),
                        }
                    },
                    Err(e) => println!("Could not load blockchain: {}", e),
                }
            } else {
                println!("Blockchain, public key or secret key file doesn't exist, want to generate a test file? (y/N)");
                generate_test_binary();
            }
        }
        _ => {
            println!("Blockchain, public key or secret key file arguments not supplied, want to generate a test file? (y/N)");
            generate_test_binary();
        }
    }
}

fn generate_test_binary() {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let ans = String::from(input.trim());
            if ans == "y" || ans == "Y" {
                println!(
                    "Ok, generating test file 'celestium.bin' with test keys in folder 'keys'"
                );
                create_test_blockchain(String::from("celestium.bin"));
            } else {
                println!("Ok, bye")
            }
        }
        Err(e) => println!("{}", e),
    }
}

fn create_test_blockchain(location: String) {
    // Create test blockchain
    let pk1_location = "keys/key1.pub";
    let sk1_location = "keys/key1";
    let pk2_location = "keys/key2.pub";
    let sk2_location = "keys/key2";

    if !PathBuf::from("keys").exists() {
        fs::create_dir("keys").unwrap();
    }

    if !PathBuf::from(pk1_location).exists() || !PathBuf::from(sk1_location).exists() {
        generate_ec_keys(PathBuf::from(pk1_location), PathBuf::from(sk1_location));
    }
    if !PathBuf::from(pk2_location).exists() || !PathBuf::from(sk2_location).exists() {
        generate_ec_keys(PathBuf::from(pk2_location), PathBuf::from(sk2_location));
    }

    let transaction1 = Transaction::new(
        UniversalId::new(false, false, 0),
        Wallet::load_public_key_from_file(&PathBuf::from(pk1_location)).unwrap(),
        Wallet::load_public_key_from_file(&PathBuf::from(pk1_location)).unwrap(),
        TransactionValue::new(10000, Some(0)),
    );
    let transaction2 = Transaction::new(
        UniversalId::new(true, false, 1),
        Wallet::load_public_key_from_file(&PathBuf::from(pk1_location)).unwrap(),
        Wallet::load_public_key_from_file(&PathBuf::from(pk2_location)).unwrap(),
        TransactionValue::new(500, Some(25)),
    );
    let transaction3 = Transaction::new(
        UniversalId::new(false, false, 2),
        Wallet::load_public_key_from_file(&PathBuf::from(pk1_location)).unwrap(),
        Wallet::load_public_key_from_file(&PathBuf::from(pk2_location)).unwrap(),
        TransactionValue::new(200, Some(10)),
    );
    let transaction4 = Transaction::new(
        UniversalId::new(false, false, 3),
        Wallet::load_public_key_from_file(&PathBuf::from(pk1_location)).unwrap(),
        Wallet::load_public_key_from_file(&PathBuf::from(pk2_location)).unwrap(),
        TransactionValue::new(500, Some(25)),
    );
    let transaction5 = Transaction::new(
        UniversalId::new(false, false, 4),
        Wallet::load_public_key_from_file(&PathBuf::from(pk1_location)).unwrap(),
        Wallet::load_public_key_from_file(&PathBuf::from(pk2_location)).unwrap(),
        TransactionValue::new(1000, Some(30)),
    );

    let mut transaction_block1: TransactionBlock = TransactionBlock::new(vec![transaction1], 1);
    let mut transaction_block2: TransactionBlock =
        TransactionBlock::new(vec![transaction2, transaction3], 1);
    let mut transaction_block3: TransactionBlock = TransactionBlock::new(vec![transaction4], 1);
    let mut transaction_block4: TransactionBlock = TransactionBlock::new(vec![transaction5], 1);

    transaction_block1.sign(PathBuf::from(sk1_location));
    transaction_block2.sign(PathBuf::from(sk1_location));
    transaction_block3.sign(PathBuf::from(sk1_location));
    transaction_block4.sign(PathBuf::from(sk1_location));

    let block0 = Block::new(
        vec![transaction_block1],
        UniversalId::new(false, true, 2),
        BlockHash::new(0),
        Wallet::load_public_key_from_file(&&PathBuf::from(pk1_location)).unwrap(),
        vec![0x13, 0x37],
    );
    let block1 = Block::new(
        vec![transaction_block2, transaction_block3, transaction_block4],
        UniversalId::new(false, true, 4),
        BlockHash::new(0x60dc7ca4),
        Wallet::load_public_key_from_file(&&PathBuf::from(pk1_location)).unwrap(),
        vec![0x41, 0x41, 0x41, 0x41],
    );
    let mut users = HashMap::new();
    users.insert(
        Wallet::load_public_key_from_file(&PathBuf::from(pk1_location)).unwrap(),
        User::new(Wallet::load_public_key_from_file(&PathBuf::from(pk1_location)).unwrap()),
    );

    let mut blockchain = Blockchain::new(vec![block0, block1], users);

    // Serialize and save blockchain to file
    let mut serialized = [0; 1000];
    let mut i = 0;
    match blockchain.serialize_into(&mut serialized, &mut i) {
        Ok(_) => {
            println!(
                "Block created from parameters and verified, saving to '{}'",
                location
            );

            let mut f = File::create(location).unwrap();
            f.write_all(&serialized[0..i].to_vec()).unwrap();
            drop(f);
        }
        Err(e) => println!("Block creation error: {}", e),
    };
}

fn generate_ec_keys(pk_file_location: PathBuf, sk_file_location: PathBuf) {
    let secp = Secp256k1::new();
    let mut rng = OsRng::new().expect("OsRng");
    let (sk, pk) = secp.generate_keypair(&mut rng);
    let mut sk_file = File::create(sk_file_location).unwrap();
    sk_file.write_all(sk.as_ref()).unwrap();
    let mut pk_file = File::create(pk_file_location).unwrap();
    pk_file.write_all(&pk.serialize()).unwrap();
}

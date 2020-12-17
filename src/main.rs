use celestium::{
    block::{Block, BlockHash},
    blockchain::Blockchain,
    serialize::Serialize,
    transaction::{Transaction, TransactionBlock, TransactionValue},
    user_id::UserId,
    wallet::Wallet,
};
use rand::rngs::OsRng;
use secp256k1::Secp256k1;
use std::{
    fs::{self, File},
    io::{self, Write},
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
    match args.blockchain_path {
        Some(bp) => {
            if bp.exists() {
                match Wallet::from_binary(bp, args.pk_path, args.sk_path) {
                    Ok(_) => loop {
                        let mut input = String::new();
                        match io::stdin().read_line(&mut input) {
                            Ok(_) => {
                                let command = String::from(input.trim());
                                if command == String::from("balance") {
                                    todo!();
                                // match w.get_balance() {
                                //     Ok(b) => println!("User balance: {}", b),
                                //     Err(e) => println!("Error while getting balance: {}", e),
                                // }
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
                println!("Blockchain file doesn't exist, want to generate a test file? (y/N)");
                generate_test_binary();
            }
        }
        None => {
            println!("Blockchain file argument not supplied, want to generate a test file? (y/N)");
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
        UserId::new(true, false, 0x341),
        Wallet::load_public_key_from_file(&PathBuf::from(pk1_location)).unwrap(),
        Wallet::load_public_key_from_file(&PathBuf::from(pk2_location)).unwrap(),
        TransactionValue::new(400, Some(10)),
    );
    let transaction2 = Transaction::new(
        UserId::new(false, false, 0x341),
        Wallet::load_public_key_from_file(&PathBuf::from(pk2_location)).unwrap(),
        Wallet::load_public_key_from_file(&PathBuf::from(pk1_location)).unwrap(),
        TransactionValue::new(500, Some(25)),
    );

    println!(
        "T1+T2 serialized len: {}",
        transaction1.serialized_len().unwrap() + transaction2.serialized_len().unwrap()
    );
    let mut transaction_block: TransactionBlock =
        TransactionBlock::new(vec![transaction1, transaction2], 2);

    transaction_block.sign(PathBuf::from(sk1_location));
    transaction_block.sign(PathBuf::from(sk2_location));

    println!(
        "TB serialized len: {}",
        transaction_block.serialized_len().unwrap()
    );

    let block = Block::new(
        vec![transaction_block],
        UserId::new(false, true, 2),
        BlockHash::new(0),
        Wallet::load_public_key_from_file(&&PathBuf::from(pk1_location)).unwrap(),
        vec![0x13, 0x37],
    );
    let mut blockchain = Blockchain::new(vec![block]);

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

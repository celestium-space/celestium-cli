use celestium::{
    block::Block,
    serialize::{DynamicSized, Serialize},
    wallet::{Wallet, DEFAULT_N_THREADS, DEFAULT_PAR_WORK},
};
use std::{fs::File, io::prelude::*};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    serialized_block_location: Option<std::path::PathBuf>,
}

fn main() {
    let args = Cli::from_args();
    match args.serialized_block_location {
        Some(serialized_block_location) => match File::open(serialized_block_location) {
            Ok(mut file) => {
                let mut serialized_block = Vec::new();
                file.read_to_end(&mut serialized_block).unwrap();
                let (pk, sk) = Wallet::generate_ec_keys();
                let wallet = Wallet::default_miner(pk, sk);
                match Block::from_serialized(&serialized_block, &mut 0) {
                    Ok(block) => {
                        println!("Initial block magic: {}", block.magic);
                        let mined_block =
                            wallet.mine_block(DEFAULT_N_THREADS, DEFAULT_PAR_WORK, *block);
                        match mined_block {
                            Some(mined_block) => {
                                println!("Post mining block magic: {}", mined_block.magic);
                                let mut serialized_block = vec![0u8; mined_block.serialized_len()];
                                mined_block
                                    .serialize_into(&mut serialized_block, &mut 0)
                                    .unwrap();
                                println!("Got block: {:x?}", serialized_block);
                            }
                            None => println!("Got none block"),
                        }
                    }
                    Err(e) => println!("Invalid block: {}", e),
                };
            }
            Err(e) => {
                println!("Error opening file: {}", e);
            }
        },
        None => println!("Please provide path to binary serialized block"),
    }
}

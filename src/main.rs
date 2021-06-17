use celestium::{
    block::Block,
    block_hash::BlockHash,
    serialize::{DynamicSized, Serialize},
    wallet::{Wallet, DEFAULT_N_THREADS, DEFAULT_PAR_WORK},
};
use std::fs::OpenOptions;
use std::{fs::File, io::prelude::*};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short)]
    command: String,
    #[structopt(short, parse(from_os_str))]
    serialized_block_location: Option<std::path::PathBuf>,
}

fn main() {
    let args = Cli::from_args();
    if args.command == "generate" {
        match Wallet::generate_init_blockchain_unmined(10) {
            Ok(blocks) => {
                println!("Generated {} blocks, serializing", blocks.len());
                let mut blocks_file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open("blocks")
                    .expect("Error opening file");

                let mut serialized_blocks_len = 0;
                for block in &blocks {
                    serialized_blocks_len += block.serialized_len();
                }

                let mut i = 0;
                let mut serialized_blocks = vec![0u8; serialized_blocks_len];
                let mut j = 0;
                for block in blocks {
                    block
                        .serialize_into(&mut serialized_blocks, &mut i)
                        .expect(&format!("Error: Could not serialize block {}", j));
                    j += 1;
                }
                blocks_file
                    .write_all(&serialized_blocks)
                    .expect("Error: Could not write to file");
            }
            Err(e) => {
                println!("Error generating blocks: {}", e);
            }
        }
    } else if args.command == "mine" {
        match args.serialized_block_location {
            Some(serialized_block_location) => match File::open(serialized_block_location) {
                Ok(mut file) => {
                    let mut serialized_blocks = Vec::new();
                    file.read_to_end(&mut serialized_blocks).unwrap();
                    let (pk, sk) = Wallet::generate_ec_keys();
                    let wallet = Wallet::default_miner(pk, sk);
                    let mut blocks = Vec::default();
                    let mut blocks_len = 0;
                    let mut i = 0;
                    while i < serialized_blocks.len() {
                        match Block::from_serialized(&serialized_blocks, &mut i) {
                            Ok(block) => blocks.push(*block),
                            Err(s) => println!("Got invalid block: {}", s),
                        };
                        blocks_len += 1;
                    }
                    let mut j = 1;
                    let mut back_hash = blocks[0].back_hash.clone();
                    let mut mined_blocks = Vec::default();
                    let mut serialized_blocks_len = 0;
                    println!("Found {} blocks, starting mining", blocks_len);
                    for mut block in blocks {
                        print!("Mining block {}/{}", j, blocks_len);
                        j += 1;
                        block.back_hash = back_hash;
                        let block_hash = block.hash();
                        let mined_block =
                            wallet.mine_block(DEFAULT_N_THREADS, DEFAULT_PAR_WORK, block);
                        back_hash = BlockHash::from(block_hash);
                        match mined_block {
                            Some(mined_block) => {
                                serialized_blocks_len += mined_block.serialized_len();
                                mined_blocks.push(mined_block);
                            }
                            None => println!("Got none block"),
                        }
                        println!(". Done");
                    }

                    i = 0;
                    j = 0;
                    let mut serialized_blocks = vec![0u8; serialized_blocks_len];
                    for block in mined_blocks {
                        block
                            .serialize_into(&mut serialized_blocks, &mut i)
                            .expect(&format!("Error: Could not serialize block {}", j));
                    }
                    let file_name = "mined_blocks";
                    println!(
                        "All blocks mined, saving {}B to {}",
                        serialized_blocks_len, file_name
                    );
                    let mut mined_blocks_file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(file_name)
                        .expect("Error opening file");

                    mined_blocks_file
                        .write_all(&serialized_blocks)
                        .expect("Error: Could not write to file");
                }
                Err(e) => {
                    println!("Error opening file: {}", e);
                }
            },
            None => println!("Please provide path to binary serialized block"),
        }
    }
}

use celestium::{
    block::Block,
    block_hash::BlockHash,
    serialize::{DynamicSized, Serialize},
    wallet::{Wallet, DEFAULT_N_THREADS, DEFAULT_PAR_WORK},
};
use colored::*;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::time::Instant;
use std::{fs::remove_file, fs::File, io::prelude::*};
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
                remove_file("blocks")
                    .unwrap_or_else(|e| println!("Warning: Could not clean file. {}", e));
                let mut f = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open("blocks")
                    .expect("Error: Could not open file");
                f.write_all(&serialized_blocks)
                    .expect("Error: Could not write to file");
                f.flush().expect("Error: Could not flush file");
                println!("Done.")
            }
            Err(e) => {
                println!("Error generating blocks: {}", e);
            }
        }
    } else if args.command == "mine" {
        match args.serialized_block_location {
            Some(serialized_block_location) => {
                let mut file = OpenOptions::new()
                    .read(true)
                    .open(serialized_block_location.clone())
                    .unwrap();
                let mut unmined_serialized_blocks = Vec::new();
                file.read_to_end(&mut unmined_serialized_blocks).unwrap();

                let (pk, sk) = Wallet::generate_ec_keys();
                let wallet = Wallet::default_miner(pk, sk);
                let mut unmined_blocks = Vec::default();
                let mut total_blocks = 0;
                let mut i = 0;
                let mut unmined_serialized_blocks_len = unmined_serialized_blocks.len();
                while i < unmined_serialized_blocks.len() {
                    match Block::from_serialized(&unmined_serialized_blocks, &mut i) {
                        Ok(block) => unmined_blocks.push(*block),
                        Err(s) => {
                            println!("Got invalid block at {}. {}", total_blocks, s);
                            break;
                        }
                    };
                    total_blocks += 1;
                }
                let mut back_hash = unmined_blocks[0].back_hash.clone();
                let mut mined_blocks = Vec::default();
                let mut mined_serialized_blocks_len = 0;
                println!(
                    "Found {} blocks ({}B), starting mining",
                    total_blocks,
                    unmined_serialized_blocks.len()
                );
                for (n, mut block) in unmined_blocks.clone().iter_mut().enumerate() {
                    if BlockHash::contains_enough_work(&block.hash()) {
                        println!(
                            "{}",
                            format!("Block {}/{} already mined ✔️", n, total_blocks).green(),
                        );
                        continue;
                    }
                    print!("Mining block {}/{}", n, total_blocks);
                    io::stdout().flush().unwrap();
                    block.back_hash = back_hash;
                    let block_hash = block.hash();
                    let start = Instant::now();
                    let mined_block =
                        wallet.mine_block(DEFAULT_N_THREADS, DEFAULT_PAR_WORK, block.clone());
                    back_hash = BlockHash::from(block_hash);
                    match mined_block {
                        Some(mined_block) => {
                            mined_serialized_blocks_len += mined_block.serialized_len();
                            unmined_serialized_blocks_len -= block.serialized_len();
                            mined_blocks.push(mined_block);
                            println!("{}", ". Done ✔️".green());
                        }
                        None => println!(". Got none block"),
                    }
                    println!("Time: {:?}", start.elapsed());

                    let mut len = 0;
                    let mut all_blocks_serialized =
                        vec![0u8; mined_serialized_blocks_len + unmined_serialized_blocks_len];
                    for (block_n, i_block) in mined_blocks.iter_mut().enumerate() {
                        i_block
                            .serialize_into(&mut all_blocks_serialized, &mut len)
                            .unwrap_or_else(|e| {
                                panic!("Error: Could not serialize block {}. {}", block_n, e)
                            });
                    }

                    for (block_n, i_block) in unmined_blocks[n + 1..].iter().enumerate() {
                        i_block
                            .serialize_into(&mut all_blocks_serialized, &mut len)
                            .unwrap_or_else(|e| {
                                panic!(
                                    "Error: Could not serialize block {}. {}",
                                    n + 1 + block_n,
                                    e
                                )
                            });
                    }
                    remove_file(serialized_block_location.clone()).unwrap_or_else(|e| {
                        println!(
                            "Warning: Could not clean \"{:?}\". {}",
                            serialized_block_location.clone(),
                            e
                        )
                    });
                    println!(
                        "Saving checkpoint ({}B) to {:?}",
                        mined_serialized_blocks_len + unmined_serialized_blocks_len,
                        serialized_block_location.clone()
                    );

                    let mut f = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(serialized_block_location.clone())
                        .unwrap();
                    f.write_all(&all_blocks_serialized).unwrap();
                    f.flush().unwrap();
                }
            }
            None => println!("Please provide path to binary serialized block"),
        }
    } else if args.command == "test" {
        match args.serialized_block_location {
            Some(serialized_block_location) => match File::open(serialized_block_location) {
                Ok(mut file) => {
                    let mut serialized_blocks = Vec::new();
                    file.read_to_end(&mut serialized_blocks).unwrap();
                    let mut i = 0;
                    let mut j = 0;
                    while i < serialized_blocks.len() {
                        print!("Block {} ", j);
                        j += 1;
                        match Block::from_serialized(&serialized_blocks, &mut i) {
                            Ok(block) => {
                                if BlockHash::contains_enough_work(&block.hash()) {
                                    println!("{}", "contains enough work ✔️".green())
                                } else {
                                    println!("{}", "does not contain enough work ❌".red())
                                }
                            }
                            Err(s) => println!("{}", s.red()),
                        };
                    }
                }
                Err(e) => {
                    println!("Error opening file: {}", e);
                }
            },
            None => println!("Please provide path to binary serialized block"),
        }
    }
}

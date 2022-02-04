use celestium::{
    block::Block,
    block_hash::BlockHash,
    serialize::{DynamicSized, Serialize},
    wallet::{BinaryWallet, Wallet, DEFAULT_N_THREADS, DEFAULT_PAR_WORK, HASH_SIZE},
};
#[macro_use]
extern crate clap;
use colored::*;
use image::{io::Reader as ImageReader, GenericImageView, ImageFormat, Rgba, RgbaImage};
use probability::{self, distribution::Sample};
use std::io::{self, Write};
use std::time::Instant;
use std::{
    cmp::{max, min},
    fs::read,
};

use indicatif::{ProgressBar, ProgressIterator};
use std::{
    env,
    fs::{self, OpenOptions},
};
use std::{fs::remove_file, fs::File, io::prelude::*};

macro_rules! unwrap_or_print {
    ($result: expr, $format_string: expr) => {
        match $result {
            Ok(r) => r,
            Err(e) => {
                println!($format_string, e);
                return;
            }
        }
    };
}

fn diff(r: (u8, u8)) -> u32 {
    if r.0 > r.1 {
        (r.0 - r.1) as u32
    } else {
        (r.1 - r.0) as u32
    }
}

fn main() {
    let matches = clap_app!(myapp =>
        (author: "Artificial Mind A/S <jhs@artificialmind.ai>")
        (about: "Celestium Command Line Interface")
        (@subcommand generate =>
            (about: "Generates a new test blockchain")
            (@arg blocks: +required +takes_value -b --blocks "Path to save binary blocks file to")
            (@arg sk: +required +takes_value -s --secret "Path to save secret key file to")
            (@arg count: +required +takes_value -c --count "Amount of unmined blocks to generate")
        )
        (@subcommand random =>
            (about: "Generates random z-vectors from noisy images")
            (@arg images: +required +takes_value -i --images "Path to directory containing noisy images")
            (@arg output: +required +takes_value -o --output "Path to location to save binary file containing random z-vectors")
            (@arg count: +required +takes_value -c --count "Amount of z-vectors to generate")
            (@arg size: +required +takes_value -s --size "Size of z-vectors to generate")
        )
        (@subcommand mine =>
            (about: "Mines a binary blocks file")
            (@arg blocks: +required +takes_value -b --blocks "Path to binary blocks file")
        )
        (@subcommand test =>
            (about: "Tests a binary blocks file for completed work")
            (@arg blocks: +required +takes_value -b --blocks "Path to binary blocks file")
        )
        (@subcommand count =>
            (about: "Count IDs")
            (@arg data: +required +takes_value -i --data "Path to data dir")
        )
    )
    .get_matches();

    if let Some(matches) = matches.subcommand_matches("generate") {
        match Wallet::generate_init_blockchain() {
            Ok(wallet) => {
                println!("Generated {} blocks, serializing", wallet.count_blocks());
                let serialized_blocks = wallet.serialize_blockchain().unwrap();
                let sk = wallet.get_sk().unwrap();
                let blocks_path = matches.value_of("blocks").unwrap();
                let sk_path = matches.value_of("sk").unwrap();
                remove_file(blocks_path)
                    .unwrap_or_else(|e| println!("Warning: Could not clean blocks file. {}", e));
                remove_file(sk_path).unwrap_or_else(|e| {
                    println!("Warning: Could not clean secret key file. {}", e)
                });
                let mut blocks_f = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(blocks_path)
                    .expect("Error: Could not create blocks file");
                let mut sk_f = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(sk_path)
                    .expect("Error: Could not create secret key file");
                blocks_f
                    .write_all(&serialized_blocks)
                    .expect("Error: Could not write to blocks file");
                println!("{:?}", sk);
                sk_f.write_all(sk.to_string().as_bytes())
                    .expect("Error: Could not write to file");
                blocks_f.flush().expect("Error: Could not flush file");
                println!("Done.")
            }
            Err(e) => {
                println!("Error generating blocks: {}", e);
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("random") {
        let random_images_location = matches.value_of("images").unwrap();
        let dir: Vec<_> = unwrap_or_print!(
            fs::read_dir(random_images_location),
            "Could not open dir: {}"
        )
        .collect();
        println!("Found {} images", dir.len());
        let image_width = 1280usize; // TODO: Get from images
        let image_height = 720usize; // TODO: Get from images
        let mut all_img_pixels: Vec<Vec<Vec<Rgba<u8>>>> =
            vec![vec![Vec::new(); image_width]; image_height];
        for entry in dir {
            let entry = unwrap_or_print!(entry, "Could not read entry: {}").path();
            println!("Parsing image {:?}", entry);
            let img = unwrap_or_print!(ImageReader::open(entry), "Courd not open image: {}")
                .decode()
                .unwrap();
            for (y, inner) in all_img_pixels.iter_mut().enumerate().take(image_width) {
                for (x, pixel) in inner.iter_mut().enumerate().take(image_height) {
                    pixel.push(img.get_pixel(x as u32, y as u32));
                }
            }
        }
        let mut img = RgbaImage::new(image_width as u32, image_height as u32);
        let mut diffs: Vec<(u32, u32, u32, u32, u16, u16)> =
            vec![(0, 0, 0, 0, 0, 0); image_width * image_height];
        for (y, inner) in all_img_pixels.iter().enumerate() {
            for (x, pixels) in inner.iter().enumerate() {
                let mut r_total_diff = 0;
                let mut g_total_diff = 0;
                let mut b_total_diff = 0;
                let mut a_total_diff = 0;
                for pixel1 in pixels {
                    for pixel2 in pixels {
                        if pixel1 != pixel2 {
                            r_total_diff += diff((pixel1.0[0], pixel2.0[0]));
                            g_total_diff += diff((pixel1.0[1], pixel2.0[1]));
                            b_total_diff += diff((pixel1.0[2], pixel2.0[2]));
                            a_total_diff += diff((pixel1.0[3], pixel2.0[3]));
                        }
                    }
                }
                diffs[x + y * image_width] = (
                    r_total_diff,
                    g_total_diff,
                    b_total_diff,
                    a_total_diff,
                    x as u16,
                    y as u16,
                );
            }
        }

        diffs.sort_by(|&(r0, g0, b0, a0, _, _), &(r1, g1, b1, a1, _, _)| {
            ((r1 as usize) + (g1 as usize) + (b1 as usize) + (a1 as usize))
                .partial_cmp(&((r0 as usize) + (g0 as usize) + (b0 as usize) + (a0 as usize)))
                .unwrap()
        });

        let count = value_t!(matches.value_of("count"), usize).unwrap_or_else(|e| {
            println!("Could not convert count param: {}", e);
            e.exit();
        });

        let top_diffs = &diffs[..min(count, diffs.len())];

        let mut max_r_diff = u32::MIN;
        let mut min_r_diff = u32::MAX;
        let mut max_g_diff = u32::MIN;
        let mut min_g_diff = u32::MAX;
        let mut max_b_diff = u32::MIN;
        let mut min_b_diff = u32::MAX;
        let mut max_a_diff = u32::MIN;
        let mut min_a_diff = u32::MAX;

        for (r, g, b, a, _, _) in top_diffs {
            max_r_diff = max(max_r_diff, *r);
            min_r_diff = min(min_r_diff, *r);
            max_g_diff = max(max_g_diff, *g);
            min_g_diff = min(min_g_diff, *g);
            max_b_diff = max(max_b_diff, *b);
            min_b_diff = min(min_b_diff, *b);
            max_a_diff = max(max_a_diff, *a);
            min_a_diff = min(min_a_diff, *a);
        }

        let mut normalized_diffs: Vec<(u8, u8, u8, u8, u16, u16)> =
            vec![(0, 0, 0, 0, 0, 0); top_diffs.len()];
        for (i, (r, g, b, _, x, y)) in top_diffs.iter().enumerate() {
            let r = (((*r - min_r_diff) as f64 / (max_r_diff as f64)) * 255.0) as u8;
            let g = (((*g - min_g_diff) as f64 / (max_g_diff as f64)) * 255.0) as u8;
            let b = (((*b - min_b_diff) as f64 / (max_b_diff as f64)) * 255.0) as u8;
            let a = 255u8;
            img.put_pixel(*x as u32, *y as u32, Rgba([r, g, b, a]));
            normalized_diffs[i] = (r, g, b, a, *x, *y);
        }
        img.save_with_format("output.png", ImageFormat::Png)
            .unwrap();

        let size = value_t!(matches.value_of("size"), usize).unwrap_or_else(|e| {
            println!("Could not convert size param: {}", e);
            e.exit();
        });
        println!("Generating {} z-vectors", count);
        let distribution = probability::distribution::Gaussian::new(0.0, 1.0);
        let z_vectors: Vec<Vec<_>> = normalized_diffs
            .iter()
            .map(|diff| {
                let mut seed = probability::source::Default::new().seed([
                    ((diff.0 as u64) << 24)
                        + ((diff.1 as u64) << 16)
                        + ((diff.2 as u64) << 8)
                        + (diff.3 as u64),
                    ((diff.4 as u64) << 16) + (diff.5 as u64),
                ]);
                (0..size)
                    .map(move |_| distribution.sample(&mut seed))
                    .collect()
            })
            .collect();

        let mut bin = vec![0u8; count * size * 8 + 4 * count];
        println!("Total len {}", bin.len());
        let mut i = 0;
        for z_vector in z_vectors {
            for sample in z_vector {
                bin[i..i + 8].copy_from_slice(&sample.to_be_bytes());
                i += 8;
            }
        }
        println!("Z-Vector cut off, coords from here on: {}", i);
        for (x, y) in normalized_diffs[..count]
            .iter()
            .map(|diff| (diff.4, diff.5))
            .collect::<Vec<(u16, u16)>>()
        {
            bin[i] = (x >> 8) as u8;
            bin[i + 1] = (x & 0xff) as u8;
            bin[i + 2] = (y >> 8) as u8;
            bin[i + 3] = (y & 0xff) as u8;
            i += 4;
        }
        let output_path = matches.value_of("output").unwrap();
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .open(output_path)
            .unwrap();
        println!("Saving {} z-vectors to '{}'", count, output_path);
        f.write_all(&bin).unwrap();
        f.flush().unwrap();
    } else if let Some(matches) = matches.subcommand_matches("mine") {
        // let data_dir = matches.value_of("blocks").unwrap();

        // let mut unmined_blocks = Vec::default();
        // let mut total_blocks = 0;
        // let mut i = 0;
        // let mut unmined_serialized_blocks_len = unmined_serialized_blocks.len();
        // let mut end_magic = Vec::new();
        // while i < unmined_serialized_blocks.len() {
        //     if unmined_serialized_blocks[i] == 0x41
        //         && unmined_serialized_blocks[i + 1] == 0x41
        //         && unmined_serialized_blocks[i + 2] == 0x41
        //         && unmined_serialized_blocks[i + 3] == 0x41
        //     {
        //         println!(
        //             "Got blocks end at byte {}-{} ({:x?})",
        //             i,
        //             i + 4,
        //             &unmined_serialized_blocks[i..i + 4]
        //         );
        //         end_magic = unmined_serialized_blocks[i..].to_vec();
        //         unmined_serialized_blocks_len = i;
        //         break;
        //     }
        //     match Block::from_serialized(&unmined_serialized_blocks, &mut i) {
        //         Ok(block) => unmined_blocks.push(*block),
        //         Err(s) => {
        //             println!("Got invalid block at {}. {}", total_blocks, s);
        //             break;
        //         }
        //     };
        //     total_blocks += 1;
        // }
        // let mut back_hash = unmined_blocks[0].back_hash.clone();
        // let mut mined_blocks = Vec::default();
        // let mut mined_serialized_blocks_len = 0;
        // println!(
        //     "Found {} blocks ({}B), starting mining",
        //     total_blocks, unmined_serialized_blocks_len
        // );
        // for (n, mut block) in unmined_blocks.clone().iter_mut().enumerate() {
        //     if BlockHash::contains_enough_work(&block.hash().hash()) {
        //         println!(
        //             "{}",
        //             format!("Block {}/{} already mined ✔️", n + 1, total_blocks).green(),
        //         );
        //         mined_blocks.push(block.clone());
        //         continue;
        //     }
        //     print!("Mining block {}/{}", n + 1, total_blocks);
        //     io::stdout().flush().unwrap();
        //     block.back_hash = back_hash;
        //     let block_hash = block.hash();
        //     let start = Instant::now();
        //     let mined_block = wallet.mine_block(DEFAULT_N_THREADS, DEFAULT_PAR_WORK, block.clone());
        //     back_hash = BlockHash::from(block_hash);
        //     match mined_block {
        //         Ok(mined_block) => {
        //             mined_serialized_blocks_len += mined_block.serialized_len();
        //             unmined_serialized_blocks_len -= block.serialized_len();
        //             mined_blocks.push(*mined_block.clone());
        //             println!("{}", ". Done ✔️".green());
        //         }
        //         Err(e) => println!(". Got none block. {}", e),
        //     }
        //     println!("Time: {:?}", start.elapsed());

        //     let mut len = 0;
        //     let mut all_blocks_serialized =
        //         vec![
        //             0u8;
        //             mined_serialized_blocks_len + unmined_serialized_blocks_len + end_magic.len()
        //         ];
        //     for (block_n, i_block) in mined_blocks.iter_mut().enumerate() {
        //         i_block
        //             .serialize_into(&mut all_blocks_serialized, &mut len)
        //             .unwrap_or_else(|e| {
        //                 panic!("Error: Could not serialize block {}. {}", block_n, e)
        //             });
        //     }
        //     for (block_n, i_block) in unmined_blocks[n + 1..].iter().enumerate() {
        //         i_block
        //             .serialize_into(&mut all_blocks_serialized, &mut len)
        //             .unwrap_or_else(|e| {
        //                 panic!(
        //                     "Error: Could not serialize block {}. {}",
        //                     n + 1 + block_n,
        //                     e
        //                 )
        //             });
        //     }
        //     all_blocks_serialized[len..].copy_from_slice(end_magic.as_slice());
        //     remove_file(serialized_block_location).unwrap_or_else(|e| {
        //         println!(
        //             "Warning: Could not clean \"{:?}\". {}",
        //             serialized_block_location, e
        //         )
        //     });
        //     println!(
        //         "Saving checkpoint ({}B) to {:?}",
        //         mined_serialized_blocks_len + unmined_serialized_blocks_len,
        //         serialized_block_location
        //     );

        //     let mut f = OpenOptions::new()
        //         .write(true)
        //         .create(true)
        //         .open(serialized_block_location)
        //         .unwrap();
        //     f.write_all(&all_blocks_serialized).unwrap();
        //     f.flush().unwrap();
        // }
    } else if let Some(matches) = matches.subcommand_matches("count") {
        let data_dir = matches.value_of("data").unwrap();

        let load =
            |filename: &str| read(format!("{}/{}", data_dir, filename)).map_err(|e| e.to_string());

        println!("Loading binary wallet...");
        let bin_wallet = &BinaryWallet {
            blockchain_bin: load("blockchain").unwrap(),
            pk_bin: load("pk").unwrap(),
            sk_bin: load("sk").unwrap(),
            on_chain_transactions_bin: load("on_chain_transactions").unwrap(),
            unspent_outputs_bin: load("unspent_outputs").unwrap(),
            nft_lookups_bin: load("nft_lookups").unwrap(),
            off_chain_transactions_bin: load("off_chain_transactions").unwrap(),
        };
        println!("Binary wallet loaded!");
        println!("Loading wallet...");
        let wallet = Wallet::from_binary(
            bin_wallet,
            env::var("RELOAD_UNSPENT_OUTPUTS").is_ok(),
            env::var("IGNORE_OFF_CHAIN_TRANSACTIONS").is_ok(),
        )
        .unwrap();
        println!("Wallet loaded!");

        let pb = ProgressBar::with_message(
            ProgressBar::new(wallet.unspent_outputs.len() as u64),
            "Searching...".to_string(),
        );
        let to_find =
            hex::decode("e400ce26224f3a281f12f9d044ed7ca7c819d985d07b8167d9f139483bb7da1c")
                .unwrap();
        let to_find = to_find.as_slice();
        for (pk, ts) in wallet.unspent_outputs {
            for ((b, th, i), to) in ts {
                if b == BlockHash::from([0u8; HASH_SIZE]) {
                    let t = wallet.off_chain_transactions.get(&th);
                    if let Some(t) = t {
                        if let Ok(r) = t.get_id() {
                            if r == to_find {
                                println!("FOUND! {}", t.get_outputs()[0].pk);
                            }
                        }
                    }
                }
            }
            pb.inc(1);
        }
        pb.finish();
    };
}

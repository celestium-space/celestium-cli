use celestium::{
    block::Block,
    block_hash::BlockHash,
    serialize::{DynamicSized, Serialize},
    wallet::{Wallet, DEFAULT_N_THREADS, DEFAULT_PAR_WORK},
};
#[macro_use]
extern crate clap;
use colored::*;
use image::{io::Reader as ImageReader, GenericImageView, ImageFormat, Rgba, RgbaImage};
use probability::{self, distribution::Sample};
use std::cmp::{max, min};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::time::Instant;
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

fn diff(r: (u8, u8)) -> u8 {
    if r.0 > r.1 {
        r.0 - r.1
    } else {
        r.1 - r.0
    }
}

fn main() {
    let matches = clap_app!(myapp =>
        (author: "Artificial Mind A/S <jhs@artificialmind.ai>")
        (about: "Celestium Command Line Interface")
        (@subcommand generate =>
            (about: "Generates a new test blockchain")
            (@arg blocks: +required +takes_value -b --blocks "Path to binary blocks file")
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
    )
    .get_matches();

    if let Some(matches) = matches.subcommand_matches("generate") {
        match Wallet::generate_init_blockchain_unmined(
            value_t!(matches.value_of("count"), u128).unwrap_or_else(|e| {
                println!("Could not convert count param: {}", e);
                e.exit();
            }),
        ) {
            Ok(blocks) => {
                println!("Generated {} blocks, serializing", blocks.len());

                let mut serialized_blocks_len = 0;
                for block in &blocks {
                    serialized_blocks_len += block.serialized_len();
                }

                let mut i = 0;
                let mut serialized_blocks = vec![0u8; serialized_blocks_len];
                for (j, block) in blocks.into_iter().enumerate() {
                    block
                        .serialize_into(&mut serialized_blocks, &mut i)
                        .unwrap_or_else(|_| panic!("Error: Could not serialize block {}", j));
                }
                let path = matches.value_of("blocks").unwrap();
                remove_file(path)
                    .unwrap_or_else(|e| println!("Warning: Could not clean file. {}", e));
                let mut f = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)
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
        let mut diffs: Vec<(u8, u8, u8, u8, u16, u16)> =
            vec![(0, 0, 0, 0, 0, 0); image_width * image_height];
        for (y, inner) in all_img_pixels.iter().enumerate() {
            for (x, pixels) in inner.iter().enumerate() {
                let mut r_range = (u8::MAX, u8::MIN);
                let mut g_range = (u8::MAX, u8::MIN);
                let mut b_range = (u8::MAX, u8::MIN);
                let mut a_range = (u8::MAX, u8::MIN);
                for pixel in pixels {
                    r_range = (min(r_range.0, pixel.0[0]), max(r_range.1, pixel.0[0]));
                    g_range = (min(g_range.0, pixel.0[1]), max(g_range.1, pixel.0[1]));
                    b_range = (min(b_range.0, pixel.0[2]), max(b_range.1, pixel.0[2]));
                    a_range = (min(a_range.0, pixel.0[3]), max(a_range.1, pixel.0[3]));
                }
                let r = diff(r_range);
                let g = diff(g_range);
                let b = diff(b_range);
                let a = 255 - diff(a_range);
                img.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
                diffs[x + y * image_width] = (r, g, b, a, x as u16, y as u16);
            }
        }
        img.save_with_format("output.png", ImageFormat::Png)
            .unwrap();

        diffs.sort_by(|&(r0, g0, b0, a0, _, _), &(r1, g1, b1, a1, _, _)| {
            ((r1 as usize) + (g1 as usize) + (b1 as usize) + (a1 as usize))
                .partial_cmp(&((r0 as usize) + (g0 as usize) + (b0 as usize) + (a0 as usize)))
                .unwrap()
        });
        let count = value_t!(matches.value_of("count"), usize).unwrap_or_else(|e| {
            println!("Could not convert count param: {}", e);
            e.exit();
        });
        let size = value_t!(matches.value_of("size"), usize).unwrap_or_else(|e| {
            println!("Could not convert size param: {}", e);
            e.exit();
        });
        println!("Generating {} z-vectors", count);
        let distribution = probability::distribution::Gaussian::new(0.0, 1.0);
        let z_vectors: Vec<Vec<_>> = diffs[..min(count, diffs.len())]
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
        let mut i = 0;
        for z_vector in z_vectors {
            for sample in z_vector {
                bin[i..i + 8].copy_from_slice(&sample.to_be_bytes());
                i += 8;
            }
        }
        for (x, y) in diffs[..count]
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
        let serialized_block_location = matches.value_of("blocks").unwrap();
        let mut file = OpenOptions::new()
            .read(true)
            .open(serialized_block_location)
            .unwrap();
        let mut unmined_serialized_blocks = Vec::new();
        file.read_to_end(&mut unmined_serialized_blocks).unwrap();

        let (pk, sk) = Wallet::generate_ec_keys();
        let wallet = Wallet::new(pk, sk, true).unwrap();
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
            print!("Mining block {}/{}", n + 1, total_blocks);
            io::stdout().flush().unwrap();
            block.back_hash = back_hash;
            let block_hash = block.hash();
            let start = Instant::now();
            let mined_block = wallet.mine_block(DEFAULT_N_THREADS, DEFAULT_PAR_WORK, block.clone());
            back_hash = BlockHash::from(block_hash);
            match mined_block {
                Ok(mined_block) => {
                    mined_serialized_blocks_len += mined_block.serialized_len();
                    unmined_serialized_blocks_len -= block.serialized_len();
                    mined_blocks.push(mined_block);
                    println!("{}", ". Done ✔️".green());
                }
                Err(e) => println!(". Got none block. {}", e),
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
            remove_file(serialized_block_location).unwrap_or_else(|e| {
                println!(
                    "Warning: Could not clean \"{:?}\". {}",
                    serialized_block_location, e
                )
            });
            println!(
                "Saving checkpoint ({}B) to {:?}",
                mined_serialized_blocks_len + unmined_serialized_blocks_len,
                serialized_block_location
            );

            let mut f = OpenOptions::new()
                .write(true)
                .create(true)
                .open(serialized_block_location)
                .unwrap();
            f.write_all(&all_blocks_serialized).unwrap();
            f.flush().unwrap();
        }
    } else if let Some(matches) = matches.subcommand_matches("test") {
        let serialized_block_location = matches.value_of("blocks").unwrap();
        match File::open(serialized_block_location) {
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
        }
    };
}

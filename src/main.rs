use celestium::{
    block::Block,
    block_hash::BlockHash,
    serialize::{DynamicSized, Serialize},
    transaction::Transaction,
    wallet::{
        BinaryWallet, Wallet, DEFAULT_N_THREADS, DEFAULT_PAR_WORK, DEFAULT_PROGRESSBAR_TEMPLATE,
    },
};
#[macro_use]
extern crate clap;
use colored::*;
use image::{io::Reader as ImageReader, GenericImageView, ImageFormat, Rgba, RgbaImage};

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::ClientOptions,
    sync::Client,
};
use probability::{self, distribution::Sample};
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::time::Instant;
use std::{
    cmp::{max, min},
    fs::read,
};
use std::{
    collections::HashMap,
    io::{self, Write},
};

use indicatif::{ProgressBar, ProgressStyle};
use std::{
    env,
    fs::{self, OpenOptions},
};
use std::{fs::remove_file, io::prelude::*};

static DEFAULT_MONGODB_STORE_COLLECTION_NAME: &str = "asteroids";

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

const COLOR_MAP: [[u8; 4]; 57] = [
    [0x00, 0x00, 0x00, 0xff],
    [0xe5, 0x00, 0x00, 0xff],
    [0x02, 0xbe, 0x01, 0xff],
    [0x00, 0x00, 0xea, 0xff],
    [0xf8, 0xf2, 0x08, 0xff],
    [0xfd, 0x5e, 0xf8, 0xff],
    [0x00, 0xd3, 0xdd, 0xff],
    [0xff, 0xff, 0xff, 0xff],
    [0x74, 0x15, 0xcd, 0xff],
    [0xf3, 0xc9, 0x9d, 0xff],
    [0x99, 0x99, 0x99, 0xff],
    [0xe5, 0x95, 0x00, 0xff],
    [0x00, 0x83, 0xc7, 0xff],
    [0x34, 0x71, 0x15, 0xff],
    [0x43, 0x27, 0x0a, 0xff],
    [0x86, 0x5a, 0x48, 0xff],
    // Leet h4cker colors
    [0xc5, 0x00, 0x00, 0xff],
    [0xff, 0x40, 0x40, 0xff],
    [0x00, 0x9e, 0x00, 0xff],
    [0x42, 0xfe, 0x41, 0xff],
    [0x00, 0x00, 0xca, 0xff],
    [0x40, 0x40, 0xff, 0xff],
    [0xc5, 0xb9, 0x00, 0xff],
    [0xff, 0xff, 0x40, 0xff],
    [0xdd, 0x3e, 0xd8, 0xff],
    [0xff, 0x9e, 0xff, 0xff],
    [0x00, 0xb3, 0xbd, 0xff],
    [0x40, 0xff, 0xff, 0xff],
    [0x54, 0x00, 0xad, 0xff],
    [0xb4, 0x55, 0xff, 0xff],
    [0xd3, 0xa9, 0x7d, 0xff],
    [0xff, 0xff, 0xdd, 0xff],
    [0x79, 0x79, 0x79, 0xff],
    [0xd9, 0xd9, 0xd9, 0xff],
    [0xc5, 0x75, 0x00, 0xff],
    [0xff, 0xd5, 0x40, 0xff],
    [0x00, 0x63, 0xa7, 0xff],
    [0x40, 0xc3, 0xff, 0xff],
    [0x14, 0x51, 0x00, 0xff],
    [0x74, 0xb1, 0x55, 0xff],
    [0x23, 0x07, 0x00, 0xff],
    [0x83, 0x67, 0x4a, 0xff],
    [0x66, 0x3a, 0x28, 0xff],
    [0xc6, 0x9a, 0x88, 0xff],
    [0x11, 0x11, 0x11, 0xff],
    [0x22, 0x22, 0x22, 0xff],
    [0x33, 0x33, 0x33, 0xff],
    [0x44, 0x44, 0x44, 0xff],
    [0x55, 0x55, 0x55, 0xff],
    [0x66, 0x66, 0x66, 0xff],
    [0x77, 0x77, 0x77, 0xff],
    [0x88, 0x88, 0x88, 0xff],
    [0xaa, 0xaa, 0xaa, 0xff],
    [0xbb, 0xbb, 0xbb, 0xff],
    [0xcc, 0xcc, 0xcc, 0xff],
    [0xdd, 0xdd, 0xdd, 0xff],
    [0xee, 0xee, 0xee, 0xff],
];

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct StoreItem {
    _id: ObjectId,
    // a: String,
    // A1: String,
    // A2: String,
    // A3: String,
    // ad: String,
    // albedo: Option<f64>,
    // BV: String,
    // class: String,
    // closeness: Option<i32>,
    // condition_code: String,
    // data_arc: f64,
    // diameter: Option<f64>,
    // diameter_sigma: Option<f64>,
    // DT: String,
    // dv: f64,
    // e: f64,
    // epoch: f64,
    // epoch_cal: f64,
    // epoch_mjd: f64,
    // equinox: String,
    // est_diameter: f64,
    // extent: String,
    // first_obs: String,
    full_name: String,
    // G: String,
    // GM: String,
    // H: String,
    // H_sigma: Option<f64>,
    // i: f64,
    // id: String,
    // inexact: bool,
    // IR: String,
    // K1: String,
    // K2: String,
    // last_obs: String,
    // M1: String,
    // M2: String,
    // ma: String,
    // moid: String,
    // moid_jup: String,
    // moid_ld: String,
    // n: String,
    // n_del_obs_used: String,
    // n_dop_obs_used: String,
    // n_obs_used: f64,
    // name: String,
    // neo: String,
    // om: f64,
    // orbit_id: String,
    // PC: String,
    // pdes: String,
    // per: String,
    // per_y: String,
    // pha: String,
    // prefix: String,
    price: Option<f64>,
    // producer: String,
    profit: Option<f64>,
    // prov_des: String,
    // q: f64,
    // rms: String,
    // rot_per: Option<f64>,
    // saved: f64,
    // score: f64,
    // sigma_a: String,
    // sigma_ad: String,
    // sigma_e: String,
    // sigma_i: String,
    // sigma_ma: String,
    // sigma_n: String,
    // sigma_om: String,
    // sigma_per: String,
    // sigma_q: String,
    // sigma_tp: String,
    // sigma_w: String,
    // spec: String,
    // spec_B: String,
    // spec_T: String,
    // spkid: f64,
    // t_jup: String,
    // tp: f64,
    // tp_cal: f64,
    // two_body: String,
    // UB: String,
    // w: f64,
    store_value_in_dust: String,
    id_hash: String,
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
            (@arg sk: +required +takes_value -s --sk "Path to binary Secret Key file")
        )
        (@subcommand test =>
            (about: "Tests a binary blocks file for completed work")
            (@arg blocks: +required +takes_value -b --blocks "Path to binary blocks file")
        )
        (@subcommand count =>
            (about: "Count IDs")
            (@arg data: +required +takes_value -i --data "Path to data dir")
        )
        (@subcommand piximg =>
            (about: "Creates a video from pixel transactions on the Celestium blockchain")
            (@arg FILE: +required +takes_value -i "Path to off chain transactions file")
            (@arg DIRECTORY: +required +takes_value -o "Path to save frames of video")
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
        let serialized_blocks_location = matches.value_of("blocks").unwrap();
        let serialized_sk_location = matches.value_of("sk").unwrap();

        let mut file = OpenOptions::new()
            .read(true)
            .open(serialized_blocks_location)
            .unwrap();
        let mut unmined_serialized_blocks = Vec::new();
        file.read_to_end(&mut unmined_serialized_blocks).unwrap();

        let mut file = OpenOptions::new()
            .read(true)
            .open(serialized_sk_location)
            .unwrap();
        let mut serialized_sk = Vec::new();
        file.read_to_end(&mut serialized_sk).unwrap();

        let secp = Secp256k1::new();
        let sk = *SecretKey::from_serialized(&serialized_sk, &mut 0).unwrap();
        let pk = PublicKey::from_secret_key(&secp, &sk);
        let wallet = Wallet::new(pk, sk).unwrap();

        let mut unmined_blocks = Vec::default();
        let mut total_blocks = 0;
        let mut i = 0;
        let mut unmined_serialized_blocks_len = unmined_serialized_blocks.len();
        let mut end_magic = Vec::new();
        while i < unmined_serialized_blocks.len() {
            if unmined_serialized_blocks[i] == 0x41
                && unmined_serialized_blocks[i + 1] == 0x41
                && unmined_serialized_blocks[i + 2] == 0x41
                && unmined_serialized_blocks[i + 3] == 0x41
            {
                println!(
                    "Got blocks end at byte {}-{} ({:x?})",
                    i,
                    i + 4,
                    &unmined_serialized_blocks[i..i + 4]
                );
                end_magic = unmined_serialized_blocks[i..].to_vec();
                unmined_serialized_blocks_len = i;
                break;
            }
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
            total_blocks, unmined_serialized_blocks_len
        );
        for (n, mut block) in unmined_blocks.clone().iter_mut().enumerate() {
            if BlockHash::contains_enough_work(&block.hash().hash()) {
                println!(
                    "{}",
                    format!("Block {}/{} already mined ✔️", n + 1, total_blocks).green(),
                );
                mined_blocks.push(block.clone());
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
                    mined_blocks.push(*mined_block.clone());
                    println!("{}", ". Done ✔️".green());
                }
                Err(e) => println!(". Got none block. {}", e),
            }
            println!("Time: {:?}", start.elapsed());

            let mut len = 0;
            let mut all_blocks_serialized =
                vec![
                    0u8;
                    mined_serialized_blocks_len + unmined_serialized_blocks_len + end_magic.len()
                ];
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
            all_blocks_serialized[len..].copy_from_slice(end_magic.as_slice());
            remove_file(serialized_blocks_location).unwrap_or_else(|e| {
                println!(
                    "Warning: Could not clean \"{:?}\". {}",
                    serialized_blocks_location, e
                )
            });
            println!(
                "Saving checkpoint ({}B) to {:?}",
                mined_serialized_blocks_len + unmined_serialized_blocks_len,
                serialized_blocks_location
            );

            let mut f = OpenOptions::new()
                .write(true)
                .create(true)
                .open(serialized_blocks_location)
                .unwrap();
            f.write_all(&all_blocks_serialized).unwrap();
            f.flush().unwrap();
        }
    } else if let Some(matches) = matches.subcommand_matches("count") {
        let data_dir = matches.value_of("data").unwrap();

        let mongodb_connection_string =
            env::var("MONGODB_CONNECTION_STRING").unwrap_or("mongodb://localhost/".to_string());

        let mongodb_database_name =
            env::var("MONGO_DATABASE_NAME").unwrap_or("asterank".to_string());

        let mut client_options = match ClientOptions::parse(&mongodb_connection_string) {
            Ok(c) => c,
            Err(e) => {
                println!("Could not create mongo client options: {}", e);
                return;
            }
        };

        // Manually set an option
        client_options.app_name = Some("celestium".to_string());
        // Get a handle to the cluster
        let mongodb_client = match Client::with_options(client_options) {
            Ok(c) => c,
            Err(e) => {
                println!("Could not set app name: {}", e);
                return;
            }
        };

        // Ping the server to see if you can connect to the cluster
        let database = mongodb_client.database(&mongodb_database_name);
        if let Err(e) = database.run_command(doc! {"ping": 1}, None) {
            println!(
                "Could not ping database \"{}\": {}",
                mongodb_database_name, e
            );
            return;
        };

        let store_collection_name: String = env::var("MONGODB_STORE_COLLECTION_NAME")
            .unwrap_or(DEFAULT_MONGODB_STORE_COLLECTION_NAME.to_string());
        let store_collection = database.collection::<StoreItem>(&store_collection_name);

        if database
            .list_collection_names(doc! {"name": &store_collection_name})
            .unwrap()
            .len()
            != 1
        {
            println!(
                "Could not find collection \"{}\" in database \"{}\"",
                &store_collection_name, mongodb_database_name
            );
            return;
        }

        println!(
            "MongoDB Connected successfully, collections: {:?}",
            database.list_collection_names(doc! {}).unwrap()
        );

        let mut ids_to_lookup = HashMap::new();

        for item in store_collection
            .find(doc! {"id_hash": {"$exists": true}}, None)
            .into_iter()
            .flatten()
            .flatten()
        {
            ids_to_lookup.insert(hex::decode(item.clone().id_hash).unwrap(), item.clone());
        }

        println!("TEST: {}", ids_to_lookup.len());

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
        println!("blockchain: {}", bin_wallet.blockchain_bin.len());
        println!("pk_bin: {}", bin_wallet.pk_bin.len());
        println!("sk_bin: {}", bin_wallet.sk_bin.len());
        println!(
            "on_chain_transactions_bin: {}",
            bin_wallet.on_chain_transactions_bin.len()
        );
        println!(
            "unspent_outputs_bin: {}",
            bin_wallet.unspent_outputs_bin.len()
        );
        println!("nft_lookups_bin: {}", bin_wallet.nft_lookups_bin.len());
        println!(
            "off_chain_transactions_bin: {}",
            bin_wallet.off_chain_transactions_bin.len()
        );
        println!("Loading wallet...");
        let wallet = Wallet::from_binary(
            bin_wallet,
            env::var("RELOAD_UNSPENT_OUTPUTS").is_ok(),
            env::var("RELOAD_NFT_LOOKUPS").is_ok(),
            env::var("IGNORE_OFF_CHAIN_TRANSACTIONS").is_ok(),
        )
        .unwrap();
        println!("Wallet loaded!");

        let our_pk = wallet.get_pk().unwrap();
        let mut registered_base_transaction_ids = Vec::new();
        let mut registered_transferred_ids = Vec::new();
        let mut unregistred_base_transaction_ids = Vec::new();
        let mut unregistred_transferred_ids = Vec::new();
        let mut our_registered_base_transaction_ids = Vec::new();
        let mut our_registered_transferred_ids = Vec::new();
        let mut our_unregistred_base_transaction_ids = Vec::new();
        let mut our_unregistred_transferred_ids = Vec::new();
        for (_, t) in wallet.off_chain_transactions {
            for o in t.get_outputs() {
                if let Ok(id) = o.value.get_id() {
                    match (
                        o.pk == our_pk,
                        ids_to_lookup.get(&id.to_vec()),
                        t.is_base_transaction(),
                    ) {
                        (true, Some(_), true) => our_registered_base_transaction_ids.push(id),
                        (true, Some(_), false) => our_registered_transferred_ids.push(id),
                        (true, None, true) => our_unregistred_base_transaction_ids.push(id),
                        (true, None, false) => our_unregistred_transferred_ids.push(id),
                        (false, Some(_), true) => registered_base_transaction_ids.push(id),
                        (false, Some(item), false) => registered_transferred_ids.push(item),
                        (false, None, true) => unregistred_base_transaction_ids.push(id),
                        (false, None, false) => unregistred_transferred_ids.push(id),
                    }
                }
            }
        }

        println!(
            "Deep lookup (not ours):\nregistered base transaction ids: {}\nregistered transferred ids: {}\nunregistred base transaction ids: {}\nunregistred transferred ids: {}",
            registered_base_transaction_ids.len(),
            registered_transferred_ids.len(),
            unregistred_base_transaction_ids.len(),
            unregistred_transferred_ids.len()
        );
        println!(
            "Deep lookup (ours):\nregistered base transaction ids: {}\nregistered transferred ids: {}\nunregistred base transaction ids: {}\nunregistred transferred ids: {}",
            our_registered_base_transaction_ids.len(),
            our_registered_transferred_ids.len(),
            our_unregistred_base_transaction_ids.len(),
            our_unregistred_transferred_ids.len()
        );

        let pb = ProgressBar::with_message(
            ProgressBar::new(registered_transferred_ids.len() as u64),
            "Buying items ;)".to_string(),
        );
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"),
        );
        for item in registered_transferred_ids {
            store_collection
                .update_one(
                    doc! {"_id": item._id},
                    doc! {"$set": {"state": "bought"}},
                    None,
                )
                .unwrap();
            pb.inc(1);
        }
        pb.finish();
    } else if let Some(matches) = matches.subcommand_matches("piximg") {
        let off_chain_transactions_file = matches.value_of("FILE").unwrap();
        let output = matches.value_of("DIRECTORY").unwrap();
        let load = |filename: &str| read(filename).map_err(|e| e.to_string());
        let off_chain_transactions_bin = load(off_chain_transactions_file).unwrap();
        let mut i = 0;
        let mut ii = 0;
        let mut image = RgbaImage::new(1000, 1000);
        for y in 0..1000 {
            for x in 0..1000 {
                image.put_pixel(x, y, Rgba([0xff, 0xff, 0xff, 0xff]));
            }
        }
        let mut pixel_base_messages = Vec::new();
        while i < off_chain_transactions_bin.len() {
            let t = *Transaction::from_serialized(&off_chain_transactions_bin, &mut i).unwrap();
            if let Ok(bytes) = t.get_base_transaction_message() {
                pixel_base_messages.push((bytes, format!("{}{:0>10}.png", output, ii)));
                ii += 1;
            }
        }

        let pb = ProgressBar::with_message(
            ProgressBar::new(pixel_base_messages.len() as u64),
            "Generating frames",
        );
        pb.set_style(ProgressStyle::default_bar().template(DEFAULT_PROGRESSBAR_TEMPLATE));
        for (bytes, file_name) in pixel_base_messages {
            pb.inc(1);
            let x = ((bytes[28] as usize) << 8) + (bytes[29] as usize);
            let y = ((bytes[30] as usize) << 8) + (bytes[31] as usize);
            let color = COLOR_MAP[bytes[32] as usize];
            if x < 1000 && y < 1000 {
                image.put_pixel(x as u32, y as u32, Rgba(color));
                image.save_with_format(file_name, ImageFormat::Png).unwrap();
            }
        }
        pb.finish();
        // ffmpeg -r 120 -i video/%10d.png -c:v libx265 canvas.mp4
    };
}

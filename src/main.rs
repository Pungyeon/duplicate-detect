use std::fs;
use std::env;
use std::fs::File;
// use std::io::prelude::*;
use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::error::Error;
use std::time::{SystemTime};

extern crate data_encoding;
extern crate ring;

use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};

fn sha256_digest<R: Read>(mut reader : R) -> Result<Digest, String> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

fn traverse_dir(path: String, hmap: &mut HashMap<String, String>, dupes: &mut HashMap<String, String>, total_files: &mut i64, verbose: bool) {
    let files = match fs::read_dir(path.clone()) {
        Ok(files) => files,
        Err(why) => {
            println!("error: could not read folder: {}: reason: {}", path.clone().to_string(), why.description());
            return
        },
    };

    for entry in files {
        let entry = entry.unwrap();
        let filepath = &entry.path().to_str().unwrap().to_string();
        let metadata = fs::metadata(&entry.path()).unwrap();
        let filetype = metadata.clone().file_type();


        if filetype.clone().is_file() {
            *total_files += 1;
            let file = match File::open(&entry.path()) {
                Ok(file) => file,
                Err(why) => {
                    println!("error: could not open file: {}: reason: {}", filepath, why.description());
                    continue;
                },
            };
            
            let reader = BufReader::new(file);
            let digest = sha256_digest(reader).unwrap();
            let hashstring = HEXUPPER.encode(digest.as_ref());

            if verbose {
                println!("checking file: {}", filepath.clone());
            }

            if hmap.contains_key(&hashstring) {
                let d = hmap.get(&hashstring).unwrap();
                dupes.insert(d.to_string(), filepath.clone());
            }

            hmap.insert(hashstring, filepath.clone());
        }

        if filetype.clone().is_dir() {
            traverse_dir(filepath.clone(), hmap, dupes, total_files, verbose);
        }
    }
}

fn main() {
    let mut hmap : HashMap<String, String> = HashMap::new();
    let mut dupes : HashMap<String, String> = HashMap::new();
    let mut total_files = 0;
    let now = SystemTime::now();

    let args : Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("you must provide one command line argument, to run this program:");
        println!("\tdupe_detect.exe [starting folder]");
        println!("parameters:");
        println!("\t-v: set verbose to true, to print progress");
        return 
    }

    let mut verbose = false;

    if args.len() > 2 {
        if args[2].to_string() == "-v" {
            verbose = true;
        }
    }

    traverse_dir(args[1].to_string(), &mut hmap, &mut dupes, &mut total_files, verbose);

    println!("DUPLICATES FOUND:");
    for d in dupes {
        println!("file: {}, copy: {}", d.0, d.1);
    }
    println!("TOTAL FILES TRAVERSED: {}", total_files);
    println!("ELAPSED TIME: {}s", now.elapsed().unwrap().as_secs());

}

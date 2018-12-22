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

struct FileIndex {
    hmap: HashMap<String, String>,
    dupes: HashMap<String, String>,
    count: i64,
    dupe_size: u64,
}

impl FileIndex {
    fn increment(&mut self) {
        self.count += 1;
    }

    fn duplication_size_increment(&mut self, size: u64) {
        self.dupe_size += size;
    }

    fn insert_index(&mut self, hash: String, name: String) {
        self.hmap.insert(hash, name);
    }

    fn insert_dupe(&mut self, file: String, copy: String) {
        self.dupes.insert(file, copy);
    }
    
    fn insert(&mut self, hash: String, filepath: String, filesize: u64) {
        if self.hmap.contains_key(&hash) {
            let d = self.hmap.get(&hash).unwrap();
            self.insert_dupe(d.to_string(), filepath.clone());
            self.duplication_size_increment(filesize);
        }
        self.insert_index(hash, filepath.clone());
    }
}

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

fn traverse_dir(path: String, index: &mut FileIndex, verbose: bool) {
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
            index.increment();

            let file = match File::open(&entry.path()) {
                Ok(file) => file,
                Err(why) => {
                    println!("error: could not open file: {}: reason: {}", filepath, why.description());
                    continue;
                },
            };
            
            let hashstring = HEXUPPER.encode(
                sha256_digest(BufReader::new(file)).unwrap().as_ref()
            );
            index.insert(hashstring, filepath.clone(), metadata.clone().len());

            if verbose { println!("checking file: {}", filepath.clone()) }
        }

        if filetype.clone().is_dir() {
            traverse_dir(filepath.clone(), index, verbose);
        }
    }
}

fn main() {
    let mut file_index : FileIndex = FileIndex{
        hmap : HashMap::new(),
        dupes : HashMap::new(),
        count: 0,
        dupe_size: 0,
    };

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

    traverse_dir(args[1].to_string(), &mut file_index, verbose);

    println!("DUPLICATES FOUND:");
    for d in file_index.dupes {
        println!("file: {}, copy: {}", d.0, d.1);
    }
    println!("TOTAL FILES TRAVERSED: {}", file_index.count);
    println!("TOTAL FILE SIZE: {}", file_index.dupe_size);
    println!("ELAPSED TIME: {}s", now.elapsed().unwrap().as_secs());

}

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

extern crate crypto;

use crypto::digest::Digest;
use crypto::sha1::Sha1;

fn traverse_dir(path: String, hmap: &mut HashMap<String, String>, dupes: &mut HashMap<String, String>) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let filepath = &entry.path().to_str().unwrap().to_string();
        let filename = entry.file_name().into_string().unwrap();
        let metadata = fs::metadata(&entry.path()).unwrap();
        let size =  metadata.clone().len();    
        let filetype = metadata.clone().file_type();
        let created = metadata.clone()
            .created().unwrap().elapsed().unwrap().as_secs();
        let last_modified =  metadata.clone()
            .modified().unwrap()
            .elapsed().unwrap()
            .as_secs();


        if !filetype.clone().is_dir() {
            // this should read bytes instead of string, derp
            let mut f = File::open(&entry.path()).expect("no such file");
            let mut content = String::new();
            f.read_to_string(&mut content).expect("could not read file");

            let mut hasher = Sha1::new();
            hasher.input_str(&mut content.to_string());
            let mut hashstring = hasher.result_str();            

            if hmap.contains_key(&mut hashstring) {
                let d = hmap.get(&mut hashstring).unwrap();
                dupes.insert(filepath.clone(), d.to_string());
            }

            hmap.insert(hashstring, filepath.clone());
        }

        if filetype.clone().is_dir() {
            traverse_dir(filepath.clone(), hmap, dupes);
        }

        println!(
            "filename: {}, last_modified: {}, size: {}, created: {}",
            filename.clone(), last_modified, size, created
        );
    }
}

fn main() {
    let mut hmap : HashMap<String, String> = HashMap::new();
    let mut dupes : HashMap<String, String> = HashMap::new();

    traverse_dir("one".to_string(), &mut hmap, &mut dupes);

    println!("ALL FILES:");
    for d in hmap {
        println!("{}: {}", d.0, d.1);
    }

    println!("DUPLICATES:");
    for d in dupes {
        println!("file: {}, copy: {}", d.0, d.1);
    }
}

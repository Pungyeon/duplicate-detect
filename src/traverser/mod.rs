pub(crate) mod file_index;

use error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};
use threadpool::ThreadPool;
use file_index::FileIndex;

use crate::error;

pub struct Traverser {
    workers: ThreadPool,
}

impl Traverser {
    pub fn new() -> Self {
        Traverser {
            workers: ThreadPool::new(std::thread::available_parallelism().unwrap().get()),
        }
    }

    pub fn parallel(&self, path: &str) {
        let now = SystemTime::now();
        let value = Arc::new(Mutex::new(FileIndex::new()));

        self.handle_dir(path, value.clone());

        self.workers.join();

        let traversal_time = now.elapsed().unwrap().as_secs();

        println!("DUPLICATES FOUND:");
        for (key, value) in value.lock().unwrap().files.iter() {
            if !value.duplicates.is_empty() {
                println!("(hash: {}): (file: {})", key, value.filename);
                for duplicate in value.duplicates.iter() {
                    println!("\t{}", duplicate);
                }
            }
        }
        println!("TOTAL FILES TRAVERSED: {}", value.lock().unwrap().count);
        println!("TOTAL FILE SIZE: {}", value.lock().unwrap().dupe_size);
        println!("TRAVERSAL TIME: {}s", traversal_time);
    }

    fn handle_dir(&self, path: &str, value: Arc<Mutex<FileIndex>>) {
        let entries = fs::read_dir(path).unwrap();

        for entry in entries {
            let p = entry.unwrap().path();
            if p.is_dir() {
                self.handle_dir(p.to_str().unwrap(), value.clone());
            } else {
                let value = value.clone();
                self.workers.execute(move || {
                    Traverser::handle_file(p, value)
                }
                );
            }
        }
    }

    fn handle_file(p: PathBuf, value: Arc<Mutex<FileIndex>>) {
        value.lock().unwrap().increment();

        let filepath = p.to_str().unwrap().to_string();
        let file = match File::open(&p) {
            Ok(file) => file,
            Err(err) => {
                println!("error: could not open file: {}: reason: {}", filepath, err);
                return
            }
        };

        let dig = BufReader::new(file);
        let hashstring = HEXUPPER.encode(
            sha256_digest(dig).unwrap().as_ref()
        );

        let metadata = fs::metadata(p).unwrap();
        value.lock().unwrap().insert(hashstring, filepath, metadata.len());
    }
}

fn sha256_digest<R: Read>(mut reader : R) -> Result<Digest, Error> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

use std::fs;
use std::env;
use std::fs::File;
use std::collections::{hash_map, HashMap};
use std::fmt::{Display, Formatter};
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::time::{SystemTime};
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

extern crate data_encoding;
extern crate ring;

use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};

#[derive(Debug)]
struct Error {
    msg: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error{
            msg: err.to_string(),
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error{
            msg: s,
        }
    }
}

struct DuplicateMeta {
    filename: String,
    duplicates: Vec<String>,
}

impl DuplicateMeta {
    fn new(filename: String) -> Self {
        DuplicateMeta{
            filename,
            duplicates: Vec::new(),
        }
    }
}

struct FileIndex {
    files: HashMap<String, DuplicateMeta>,
    count: i64,
    dupe_size: u64,
}

impl FileIndex {
    fn new() -> Self {
        FileIndex{
            files: HashMap::new(),
            count: 0,
            dupe_size: 0,
        }
    }

    fn increment(&mut self) {
        self.count += 1;
    }

    fn duplication_size_increment(&mut self, size: u64) {
        self.dupe_size += size;
    }

    fn insert(&mut self, hash: String, filepath: String, filesize: u64) {
        if let hash_map::Entry::Vacant(e) = self.files.entry(hash.clone()) {
            e.insert(DuplicateMeta::new(filepath));
        } else {
            self.files.get_mut(&hash).unwrap().duplicates.push(filepath);
            self.duplication_size_increment(filesize);
        }
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

struct Traverser {
    workers: ThreadPool,
}

impl Traverser {
    fn new() -> Self {
        Traverser {
            workers: ThreadPool::new(std::thread::available_parallelism().unwrap().get()),
        }
    }

    fn parallel(&self, path: &str) {
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

fn main() {
    let now = SystemTime::now();
    let args : Vec<String> = env::args().collect();

    if args.len() < 2 {
        help();
        return
    }

    let traverser = Traverser::new();

    traverser.parallel(args[1].as_str());

    println!("ELAPSED TIME: {}s", now.elapsed().unwrap().as_secs());
}

fn help() {
    println!("you must provide one command line argument, to run this program:");
    println!("\tdupe_detect.exe [starting folder]");
    println!("parameters:");
    println!("\t-v: set verbose to true, to print progress");
}
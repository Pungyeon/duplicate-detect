mod error;
mod traverser;

extern crate data_encoding;
extern crate ring;

use std::env;
use std::time::SystemTime;
use traverser::Traverser;

fn main() {
    let now = SystemTime::now();
    let args : Vec<String> = env::args().collect();

    if args.len() < 2 {
        help();
        return
    }

    Traverser::new().parallel(args[1].as_str());

    println!("ELAPSED TIME: {}s", now.elapsed().unwrap().as_secs());
}

fn help() {
    println!("you must provide one command line argument, to run this program:");
    println!("\tdupe_detect.exe [starting folder]");
    println!("parameters:");
    println!("\t-v: set verbose to true, to print progress");
}
mod mvdup;

use clap::Parser;
use core::panic;
use glob::{glob, Paths};

use std::convert::AsRef;
use std::path::Path;

#[derive(Parser, Debug)]
struct Args {
    source: String,
    destination: String,
}

fn main() {
    let args = Args::parse();

    let dst = valid_destination(args.destination.as_str());

    let srcs = get_sources(args.source);

    for src in srcs {
        match src {
            Ok(src) => {
                let src: Box<Path> = src.into();
                println!("{:?} {:?}", src.clone(), mvdup::hash::hash_of(src))
            }
            Err(e) => println!("{:?}", e),
        }
    }
}

/// Check is valid directory and return Path.
fn valid_destination<'a>(path: &'a str) -> &'a Path {
    let dst_path = Path::new(path);

    let is_dir = mvdup::fs::is_dir(dst_path);
    if let Err(err) = is_dir {
        panic!("can not open destination: {:?}", err);
    }

    let is_dir = is_dir.unwrap();
    if !is_dir {
        panic!("destination is not file directory");
    }

    dst_path
}

fn get_sources<T: AsRef<str>>(path: T) -> Paths {
    glob(path.as_ref()).expect("Failed to read glob pattern")
}

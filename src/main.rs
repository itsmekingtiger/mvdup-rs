mod mvdup;

use clap::Parser;
use core::panic;
use glob::{glob, Paths};
use mvdup::utils::StringUtils;

use std::convert::AsRef;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
struct Args {
    source: String,
    destination: String,
}

fn main() {
    let args = Args::parse();

    let dst_dir = valid_destination(args.destination.as_str());

    let srcs = get_sources(args.source);

    mvdup::database::open_at(dst_dir);

    for src in srcs {
        match src {
            Ok(src) => {
                let filename = src.file_name().expect("no filename in");

                let dst = PathBuf::from(dst_dir).with_file_name(filename);
                let hash = mvdup::hash::hash_of(&src).unwrap();
                println!("{:?} {}", src, hash.substring(0, 8));

                let (isdup, filename) = mvdup::database::is_duplicated(dst_dir, hash);

                if isdup {
                    todo!("add to temporal list")
                } else {
                    println!("moving file from {:?} to {:?}", src, dst);
                    fs::rename(src, dst).expect("failed to move file");
                }
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

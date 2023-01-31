mod mvdup;

use clap::Parser;
use core::panic;
use glob::{glob, Paths};
use mvdup::utils::StringUtils;

use std::convert::AsRef;
use std::fs;
use std::path::{Path, PathBuf};

use crate::mvdup::fs::is_regular_file;

#[derive(Parser, Debug)]
struct Args {
    /// Source files
    source: String,

    /// Destination directory
    destination: String,

    /// Process first N files only
    #[arg(long, value_name = "NUMBER OF FILES")]
    take: Option<usize>,
}

fn main() {
    let args = Args::parse();

    let dst_dir = valid_destination(args.destination.as_str());

    let mut srcs = get_sources(args.source);

    mvdup::database::open_at(dst_dir);

    let mut take_count: usize = 0;

    for src in srcs {
        match src {
            Ok(src) => {
                let filename = src.file_name().expect("no filename in");

                if is_regular_file(src.as_path()).unwrap() == false {
                    println!(
                        "{} is not regular file, will be skipped",
                        src.to_str().unwrap()
                    );
                    continue;
                }

                if let Some(take_n) = args.take {
                    if take_count >= take_n {
                        break;
                    }
                    take_count += 1;
                }

                let dst = PathBuf::from(dst_dir).join(filename);
                let hash = mvdup::hash::hash_of(&src).unwrap();
                println!("{} {}", src.to_str().unwrap(), hash.substring(0, 8));

                let (isdup, exist_filename) = mvdup::database::is_duplicated(dst_dir, hash);

                if isdup {
                    todo!("add to temporal list")
                } else {
                    println!("rename {:?} → {:?}", src, dst);
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

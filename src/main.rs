mod mvdup;

use clap::Parser;
use core::panic;
use glob::{glob, Paths};
use mvdup::utils::StringUtils;

use std::collections::HashMap;
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

struct DuplicataionManager {
    entries: HashMap<String, DuplicationEntry>,
}

impl DuplicataionManager {
    fn new() -> DuplicataionManager {
        DuplicataionManager {
            entries: HashMap::new(),
        }
    }

    fn put(&mut self, hash_value: String, exists: String, duplicated: String) {
        match self.entries.get_mut(&hash_value) {
            Some(entry) => entry.push(duplicated),
            None => self._new_entry(hash_value, exists, duplicated),
        }
    }

    fn _new_entry(&mut self, hash_value: String, exists: String, duplicated: String) {
        self.entries
            .insert(hash_value, DuplicationEntry::new(exists, duplicated));
    }
}

struct DuplicationEntry {
    on_destination: String,
    sources: Vec<String>,
}

impl DuplicationEntry {
    fn new(dst: String, src: String) -> DuplicationEntry {
        DuplicationEntry {
            on_destination: dst,
            sources: vec![src],
        }
    }

    fn push(&mut self, src: String) {
        self.sources.push(src);
    }
}

fn main() {
    let args = Args::parse();

    let dst_dir = valid_destination(args.destination.as_str());

    let srcs = get_sources(args.source);

    mvdup::database::open_at(dst_dir);

    let mut manager = DuplicataionManager::new();
    let mut take_count: usize = 0;

    for src in srcs {
        match src {
            Ok(src) => {
                let filename = src
                    .file_name()
                    .expect("no filename in")
                    .to_os_string()
                    .into_string()
                    .expect("can not convert filename into string");

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

                let dst = PathBuf::from(dst_dir).join(&filename);
                let hash = mvdup::hash::hash_of(&src).unwrap();
                println!("{} {}", src.to_str().unwrap(), hash.substring(0, 8));

                let (isdup, exist_filename) =
                    mvdup::database::is_duplicated(dst_dir, hash.as_str());

                if isdup {
                    manager.put(hash, exist_filename, String::from(src.to_str().unwrap()))
                } else {
                    println!("rename {:?} → {:?}", src, dst);
                    fs::rename(&src, &dst).expect("failed to move file");
                    mvdup::database::add(dst_dir, hash, filename);
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }

    // Resolve duplication
    for (_, entry) in manager.entries {
        println!("---");
        println!("Resolve below duplication:");
        println!(
            "\t{}",
            PathBuf::from(dst_dir)
                .join(entry.on_destination)
                .to_str()
                .unwrap()
        );
        entry.sources.iter().for_each(|src| println!("\t\t{}", src));
        println!();
    }

    // run web server

    // get request

    // do resolve
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

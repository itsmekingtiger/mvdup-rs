use core::panic;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use glob::{glob, Paths};

use crate::mvdup::{
    fs::{is_exist, is_regular_file},
    utils::StringUtils,
};

use super::{
    database,
    fs::{is_dir, list_files},
};

#[derive(Parser, Debug)]
pub struct Cli {
    /// Source files
    pub source: Option<Vec<String>>,

    /// Destination directory
    // pub destination: Option<String>,

    /// Process first N files only
    #[arg(long, value_name = "NUMBER OF FILES")]
    pub take: Option<usize>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Update files to database
    Update {
        path: String,

        /// Recalculate hash value of files
        #[arg(long)]
        verify: bool,
    },
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

pub fn mvdup(args: Cli) {
    let mut paths: Vec<String> = match args.source {
        Some(paths) => {
            if paths.len() < 2 {
                println!("give me src and dst!");
                std::process::exit(-1);
            } else {
                paths
            }
        }
        None => {
            println!("give me src and dst!");
            std::process::exit(-1);
        }
    };

    let dst_dir = paths.pop().unwrap();
    let dst_dir = valid_destination(dst_dir.as_str());

    let srcs: Vec<_> = paths.iter().map(get_sources).flatten().collect();
    for src in &srcs {
        if let Err(src) = src {
            panic!("failed to read glob pattern: {}", src);
        }
    }
    let srcs: Vec<_> = srcs.into_iter().filter_map(|e| e.ok()).collect();

    database::open_at(dst_dir);

    let mut manager = DuplicataionManager::new();
    let mut take_count: usize = 0;

    for src in srcs {
        let src = PathBuf::from(src);
        {
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
            let hash = super::hash::hash_of(&src).unwrap();
            println!("{} {}", src.to_str().unwrap(), hash.substring(0, 8));

            let (isdup, exist_filename) = super::database::is_duplicated(dst_dir, hash.as_str());

            if isdup {
                manager.put(hash, exist_filename, String::from(src.to_str().unwrap()))
            } else {
                println!("rename {:?} → {:?}", src, dst);
                if is_exist(&dst) {
                    panic!("can not move, {} is already exists!", dst.to_string_lossy());
                }
                fs::rename(&src, &dst).expect("failed to move file");
                super::database::add(dst_dir, hash, filename);
            }
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

    let is_dir = super::fs::is_dir(dst_path);
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

/// Create database file if is not exists at given path.
/// Then apply difference of file
pub fn update(dst_dir: String, verify: bool) {
    match is_dir(&dst_dir) {
        Ok(true) => (),
        Ok(false) => panic!("{dst_dir} is not valid directory."),
        Err(err) => panic!("can not determind {dst_dir} is valid directory: {err}"),
    }

    database::open_at(&dst_dir);

    // Listing files, exists and saved
    let exist_files: Vec<_> = list_files(&dst_dir)
        .expect("failed to read list of files")
        .into_iter()
        .map(|p| p.file_name().unwrap().to_os_string().into_string().unwrap())
        .collect();
    let exist_files: HashSet<_> = HashSet::from_iter(exist_files);

    let saved_files: Vec<_> = database::read_all(&dst_dir)
        .expect("failed to read database")
        .into_iter()
        .map(|f| f.0)
        .collect();
    let saved_files: HashSet<_> = HashSet::from_iter(saved_files);

    // Calculate diff
    let added = exist_files.difference(&saved_files);
    let deleted = saved_files.difference(&exist_files);

    // Resolve
    for ele in added {
        println!("+{ele}")
    }

    for ele in deleted {
        println!("-{ele}")
    }

    todo!()
}

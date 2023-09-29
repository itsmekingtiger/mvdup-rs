use core::panic;
use std::{
    collections::{HashMap, HashSet},
    env,
    path::{Path, PathBuf}
};

use clap::{Parser, Subcommand};
use glob::{glob, Paths};

use crate::mvdup::{
    fs::{filename_of, is_regular_file, move_file},
    utils::StringUtils,
};
use crate::mvdup::database::{append_db_filename, DataBase};
use crate::mvdup::fs::{extension_of, is_exist, must_is_dir};

use super::{
    fs::{is_dir, list_files},
};

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}


#[derive(Subcommand, Debug)]
pub enum Commands {
    Add {
        /// Source files
        source: Vec<String>,

        /// Destination directory
        #[arg(short, long, value_name = "DATABASE PATH")]
        destination: Option<String>,

        /// Process first N files only
        #[arg(long, value_name = "NUMBER OF FILES")]
        take: Option<usize>,
    },

    /// Update files to database
    Update {
        path: String,

        /// Recalculate hash value of files
        #[arg(long)]
        verify: bool,
    },

    /// Find entry of database
    Grep {
        /// 검색할 문자열
        target: String,

        /// 검색할 데이터베이스 경로
        path: Option<String>,
    },

    OpenTest {
        /// Path of database
        path: String,
    },

    Init {
        /// Path of database
        path: String,

        /// Password of database
        #[clap(short, long, value_name = "PASSWORD OF DATABASE", default_value_t = false)]
        encrypt: bool,
    },
}

struct DuplicationManager {
    entries: HashMap<String, DuplicationEntry>,
}

impl DuplicationManager {
    fn new() -> DuplicationManager {
        DuplicationManager {
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

pub fn handle_init(path: String, encrypt: bool) {
    if !is_exist(&path) {
        panic!("Directory does not exists!")
    }

    if !must_is_dir(&path) {
        panic!("Given path is not does not exists!")
    }

    // TODO: path could be target directory or explicit database path.
    //       So we need to test is exist, is dir or files and other things.

    let db_path = append_db_filename(&path);

    if encrypt {
        let input = rpassword::prompt_password("Set password: ").unwrap();
        let confirm = rpassword::prompt_password("Confirm password: ").unwrap();

        if input != confirm {
            panic!("Passwords are not same!");
        }

        DataBase::init(&db_path, Some(input)).expect("Failed to initialize database");
    } else {
        DataBase::init(&db_path, None).expect("Failed to initialize database");
    }

    // Just to show to user
    let mut abs_db_path = std::fs::canonicalize(&db_path)
        .expect("What?")
        .to_string_lossy()
        .to_string(); // What is difference between to_string() and into_string()?

    if cfg!(windows) {
        abs_db_path = abs_db_path
            .strip_prefix(r"\\?\")
            .expect("Shity")
            .parse()
            .unwrap();
    }

    println!("Database initialized {}", abs_db_path);
}



pub fn handle_open_test(path: String) {
    let db_path = append_db_filename(path);

    println!("Opening database {:?}", db_path);
    DataBase::try_open(db_path.as_ref()).expect("Failed to open database");
    println!("Successfully open database");
}

pub fn handle_add(
    paths: Vec<String>,
    destination: Option<String>,
    take: Option<usize>,
) {

    let dst_dir: &Path = &match destination {
        Some(dst) => PathBuf::from(dst),
        None => env::current_dir().unwrap(),
    };

    let srcs: Vec<_> = paths.iter().map(get_sources).flatten().collect();
    for src in &srcs {
        if let Err(src) = src {
            panic!("failed to read glob pattern: {}", src);
        }
    }
    let srcs: Vec<_> = srcs.into_iter().filter_map(|e| e.ok()).collect();


    let db_path = append_db_filename(dst_dir);
    let database = DataBase::try_open(db_path.as_ref()).expect("Failed to open database");

    let mut manager = DuplicationManager::new();
    let mut take_count: usize = 0;

    for src in srcs {
        let src = PathBuf::from(src);
        {
            let filename = filename_of(&src).expect("can not convert filename into string");
            let ext = extension_of(&src);

            if is_regular_file(src.as_path()).unwrap() == false {
                println!(
                    "{} is not regular file, will be skipped",
                    src.to_str().unwrap()
                );
                continue;
            }

            if let Some(take_n) = take {
                if take_count >= take_n {
                    break;
                }
                take_count += 1;
            }

            let hash = super::hash::hash_of_as_stream(&src).unwrap();
            let new_filename = [hash.clone(), ext.unwrap_or("".to_string())].join(".");
            let dst = PathBuf::from(dst_dir).join(new_filename);
            println!("{} {}", src.to_str().unwrap(), hash.substring(0, 8));

            let (is_duplicated, exist_filename) = database.is_duplicated(hash.as_str());

            if is_duplicated {
                manager.put(hash, exist_filename, String::from(src.to_str().unwrap()))
            } else {
                println!("rename {:?} → {:?}", src, dst);

                match move_file(src, dst) {
                    Ok(_) => database.add(hash, filename),
                    Err(err) => panic!("{err:?}"),
                }
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

    let mut path = PathBuf::new();
    path.push("/asdf");

    let v: &Path = &path;

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
pub fn handle_update(dst_dir: String, verify: bool) {
    match is_dir(&dst_dir) {
        Ok(true) => (),
        Ok(false) => panic!("{dst_dir} is not valid directory."),
        Err(err) => panic!("can not determined {dst_dir} is valid directory: {err}"),
    }

    let passwd = rpassword::prompt_password("Your password: ").unwrap();

    let db_path = append_db_filename(&dst_dir);
    let database = DataBase::try_open(db_path.as_ref()).expect("Failed to open database");


    // Listing files, exists and saved
    let exist_files: Vec<_> = list_files(&dst_dir, false)
        .expect("failed to read list of files")
        .into_iter()
        .map(|p| p.file_name().unwrap().to_os_string().into_string().unwrap())
        .collect();
    let exist_files: HashSet<_> = HashSet::from_iter(exist_files);

    let saved_files: Vec<_> = database.read_all()
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

pub fn handle_find<S: AsRef<str>>(dst_dir: String, target: S) {
    let passwd = rpassword::prompt_password("Your password: ").unwrap();

    let db_path = append_db_filename(dst_dir);
    let database = DataBase::try_open(db_path.as_ref()).expect("Failed to open database");

    let entries =
        database.find(target.as_ref().to_string()).expect("failed to query database");

    let total = entries.len();

    println!("total {}", total);
    for entry in entries {
        println!("{} {}", &entry.1[0..7], entry.0);
    }
}

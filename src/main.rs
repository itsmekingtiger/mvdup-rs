mod mvdup;

use core::panic;
use std::path::Path;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    source: String,
    destination: String,
}

fn main() {
    let args = Args::parse();

    let dst = valid_destination(args.destination.as_str());

    println!("Moving {} to {:?}!", args.source, dst);
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

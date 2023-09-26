use std::{fs, io};
use std::path::PathBuf;

use anyhow::Result;
use hex::encode;
use sha2::{Digest, Sha256};

/// SHA-256 value of file
pub fn hash_of_as_stream(file_path: &PathBuf) -> Result<String> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();

    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();

    return Ok(encode(hash));
}

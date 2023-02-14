use std::path::PathBuf;

use anyhow::{Context, Result};

/// SHA-256 value of file
pub fn hash_of(file_path: &PathBuf) -> Result<String> {
    let file_path = file_path.as_ref();
    sha256::try_digest(file_path).context("failed to calculate hash")
}

pub fn hash_of_as_stream(file_path: &PathBuf) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::{fs, io};

    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    let n = io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();

    let asdf = hex::encode(hash);
    return Ok(asdf);
}

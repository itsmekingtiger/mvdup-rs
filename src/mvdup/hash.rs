use std::{io::Result, path::PathBuf};

use sha256::try_digest;

/// SHA-256 value of file
pub fn hash_of(file_path: &PathBuf) -> Result<String> {
    let file_path = file_path.as_ref();
    sha256::try_digest(file_path)
}

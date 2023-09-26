use std::{io::Result, path::PathBuf};

/// SHA-256 value of file
pub fn hash_of(file_path: &PathBuf) -> Result<String> {
    sha256::try_digest(file_path)
}

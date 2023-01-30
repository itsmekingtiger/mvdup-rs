use std::{io::Result, path::Path};

use sha256::try_digest;

/// SHA-256 value of file
fn hash_of(file_path: &Path) -> Result<String> {
    sha256::try_digest::<&Path>(file_path)
}

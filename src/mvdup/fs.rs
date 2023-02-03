use std::{
    fs::{metadata, read_dir},
    io::Result,
    path::{Path, PathBuf},
};

pub fn list_files<P: AsRef<Path>>(dir_path: P) -> Result<Vec<PathBuf>> {
    let paths = read_dir(dir_path)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|e| e.is_file())
        .collect();

    Ok(paths)
}

pub fn is_regular_file<P: AsRef<Path>>(path: P) -> Result<bool> {
    let metadata = metadata(path)?;
    return Ok(metadata.is_file());
}

pub fn is_dir<P: AsRef<Path>>(path: P) -> Result<bool> {
    let metadata = metadata(path)?;
    return Ok(metadata.is_dir());
}

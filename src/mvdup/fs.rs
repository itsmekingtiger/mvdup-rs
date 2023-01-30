use std::{
    fs::{metadata, read_dir, ReadDir},
    io::Result,
    path::Path,
};

pub fn list_files(dir_path: &Path) -> Result<ReadDir> {
    read_dir(dir_path)
}

pub fn is_regular_file(path: &Path) -> Result<bool> {
    let metadata = metadata(path)?;
    return Ok(metadata.is_file());
}

pub fn is_dir(path: &Path) -> Result<bool> {
    let metadata = metadata(path)?;
    return Ok(metadata.is_dir());
}

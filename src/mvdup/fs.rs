use anyhow::{anyhow, bail, Context, Result};

use std::{
    fs::{self, metadata, read_dir},
    path::{Path, PathBuf},
};

pub fn list_files<P: AsRef<Path>>(dir_path: P, include_hidden_files: bool) -> Result<Vec<PathBuf>> {
    let paths = read_dir(dir_path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| include_hidden_files || !path.starts_with("."))
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

pub fn is_exist<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// move file from `src` to `dst`
///
/// `src` must be valid file.
/// `dst` must be valid file path or directory.
///
/// If src is not valid file, it returns error.
/// If src an dst is same, it returns error.
/// If there is same name on dst, it returns error.
pub fn move_file<P: AsRef<Path>>(src: P, dst: P) -> Result<()> {
    let dst = dst.as_ref();
    let src = src.as_ref();

    if dst.is_dir() {
        let filename =
            filename_of(src).context(format!("can not extract filename from {src:?}",))?;

        let dst = dst.join(filename);

        fs::rename(src, &dst).context(format!(
            "failed to move {:?} to {:?}, there is a file which has a same name",
            src, dst
        ))
    } else {
        if dst.exists() {
            return Err(anyhow!(
                "failed to move {:?} to {:?}, there is a file which has a same name",
                src,
                dst
            ));
        }

        fs::rename(src, dst).expect("failed to move file");

        Ok(())
    }
}

pub fn filename_of<P: AsRef<Path>>(path: P) -> Result<String> {
    let filename = path.as_ref().file_name().context("no filename in")?;

    let filename = filename.to_os_string();

    let filename = filename
        .to_str()
        .context("can not convert filename into string")?;

    Ok(filename.to_string())
}

use ctor::{ctor, dtor};
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::{fs, io};

static TEMP_DIRS: &mut Vec<PathBuf> = &mut Vec::new();

pub fn create_temp_dir<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let dir = temp_dir().join(path);
    create_dir_all(&dir)?;
    TEMP_DIRS.push(dir.clone());

    Ok(dir)
}

#[dtor]
pub fn clean_temp_dirs() -> io::Result<()> {
    for dir in TEMP_DIRS {
        fs::remove_dir_all(dir)?;
    }

    Ok(())
}
#![allow(static_mut_refs)]

use ctor::dtor;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::{fs, io};

static mut TEMP_DIRS: Vec<PathBuf> = Vec::new();

pub fn create_temp_dir<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let dir = temp_dir().join(path);
    create_dir_all(&dir)?;
    unsafe { TEMP_DIRS.push(dir.clone()); }

    Ok(dir)
}

#[dtor]
fn clean_temp_dirs() {
    unsafe {
        for dir in TEMP_DIRS.clone() {
            fs::remove_dir_all(dir).unwrap();
        }
    }
}
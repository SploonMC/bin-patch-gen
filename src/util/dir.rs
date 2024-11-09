//! Utilities for managing temporary directories.

#![allow(static_mut_refs)]

use ctor::dtor;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// List of all temporary directories in the running program.
static mut TEMP_DIRS: Vec<PathBuf> = Vec::new();

/// Creates a temporary directory with the name of [`path`] in the system's
/// temporary directory.
///
/// # Arguments
///
/// * `path` - The path which should be appended to the system's temporary directory.
///
/// # Returns
///
/// A result of the created temporary directory.
pub fn create_temp_dir<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let dir = temp_dir().join(path);
    create_dir_all(&dir)?;
    unsafe { TEMP_DIRS.push(dir.clone()); }

    Ok(dir)
}

/// Cleans up all generated temporary directories.
///
/// The [`dtor`] attribute makes this function call at the end of the program.
#[dtor]
fn clean_temp_dirs() {
    unsafe {
        for dir in TEMP_DIRS.clone() {
            fs::remove_dir_all(dir).unwrap();
        }
    }
}
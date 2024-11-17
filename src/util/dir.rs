//! Utilities for managing temporary directories.

#![allow(static_mut_refs)]

use ctor::dtor;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::{fs, io};
use std::io::{Error, ErrorKind};

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

/// Removes all contents of given directory
///
/// # Arguments
///
/// * `path` - The path to the directory, the contents of which need to be removed.
///
/// # Returns
///
/// A result that is either `Ok(())` if the contents were removed successfully, or `Err(io::Error)`
/// if an error occurred while removing the contents.
pub async fn clear_directory<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let dir = path.as_ref();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
    } else {
        Err(Error::new(ErrorKind::InvalidInput, "The provided path is not a directory"))?;
    }
    Ok(())
}

/// Cleans up all generated temporary directories.
///
/// The [`dtor`] attribute makes this function call at the end of the program.
#[dtor]
fn clean_temp_dirs() {
    unsafe {
        for dir in TEMP_DIRS.clone() {
            let _ = fs::remove_dir_all(dir);
        }
    }
}
#![allow(clippy::expect_fun_call)]

use std::{fs, io, path};
use std::fs::File;
use std::path::Path;
use zip::ZipArchive;

pub fn extract_jar<P: AsRef<Path>>(jar_file: P, output_directory: P) -> io::Result<()> {
    let file_stream = File::open(&jar_file)
        .expect(&format!("failed opening file {}", jar_file.as_ref().to_string_lossy()));
    let mut zip = ZipArchive::new(file_stream)?;

    for i in 0..zip.len() {
        let mut zip_file = zip.by_index(i)?;
        if let Some(enclosed_name) = zip_file.enclosed_name() {
            let output_path = path::absolute(output_directory.as_ref().join(enclosed_name.clone()))?;

            if zip_file.is_dir() {
                continue
            }

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut real_file = File::create(&output_path)?;
            io::copy(&mut zip_file, &mut real_file)?;
        }
    }

    Ok(())
}

pub fn has_dir<P: AsRef<Path>>(jar_file: P, file: &str) -> io::Result<bool> {
    let file_stream = File::open(&jar_file)
        .expect(&format!("failed opening file {}", jar_file.as_ref().to_string_lossy()));
    let mut zip = ZipArchive::new(file_stream)?;

    let file_name = path::absolute(file)?;
    for i in 0..zip.len() {
        let zip_file = zip.by_index(i)?;
        if let Some(enclosed_name) = zip_file.enclosed_name() {
            let absolute_path = path::absolute(enclosed_name.clone())?;
            if !zip_file.is_dir() {
                continue
            }

            if absolute_path.to_str().unwrap_or("") == file_name.to_str().unwrap_or(";;<") {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

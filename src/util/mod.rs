//! Module containing utilities.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use chrono::Local;
use sha1::{Digest, Sha1};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

pub mod dir;

pub struct TimeFormatter;
impl FormatTime for TimeFormatter {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let date = Local::now();
        write!(w, "{}", date.format("%H:%M:%S"))
    }
}

pub fn sha1<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut bytes = vec![];
    let mut file = File::open(path)?;
    file.read_to_end(&mut bytes)?;

    let mut hasher = Sha1::new();
    hasher.update(bytes);

    Ok(hex::encode(hasher.finalize()))
}

pub trait StringOptionExt<T> {
    fn equals(&self, other: T) -> bool;
}

impl StringOptionExt<String> for Option<String> {
    fn equals(&self, other: String) -> bool {
        if self.is_none() {
            return false;
        }
        
        let string = self.clone().unwrap();
        
        string == other
    }
}
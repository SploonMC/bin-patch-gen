use std::{fs, io, path::{Path, PathBuf}};

use proc_macros::serial_snake;

#[serial_snake]
pub struct Config {
    java_8_home: String,
    java_16_home: String,
    java_17_home: String,
    java_21_home: String
}

impl Config {
    pub fn java_home(&self, version: u8) -> PathBuf {
        PathBuf::from(match version {
            8 =>  &*self.java_8_home,
            16 => &*self.java_16_home,
            17 => &*self.java_17_home,
            21 => &*self.java_21_home,
            _ => panic!("invalid java home version: {version}")
        })
    }
}

pub fn read_config<P: AsRef<Path>>(path: P) -> io::Result<Config> {
    let content = fs::read_to_string(path)?;
    match toml::from_str::<Config>(&content) {
        Ok(c) => Ok(c),
        Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string()))
    }
}

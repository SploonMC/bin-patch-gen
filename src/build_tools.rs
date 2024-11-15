//! Downloads and runs BuildTools.

use std::{fs, io};
use std::io::ErrorKind;
use crate::download_url;
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;
use crate::util::dir;

/// The URL of the latest BuildTools JAR build from SpigotMC's Jenkins.
pub const VANILLA_JAR_REGEX: &str = r"(minecraft_)?server.(1\.\d{1,2}(?:\.\d{1,2})?)\.jar";
const BUILDTOOLS_URL: &str = "https://hub.spigotmc.org/jenkins/job/BuildTools/lastSuccessfulBuild/artifact/target/BuildTools.jar";
const SPIGOT_JAR_REGEX: &str = r"spigot-(1\.\d{1,2}(?:\.\d{1,2})?)\.jar";

/// Downloads the latest BuildTools from Spigot Jenkins.
///
/// # Arguments
///
/// * `path` - The path the JAR should be saved to.
///
/// # Panics
///
/// The iterator over the response stream will panic when it fails
/// to read bytes or write them to the JAR file. 
pub async fn download_buildtools<P: AsRef<Path>>(path: P) -> Result<(), reqwest::Error> {
    download_url(BUILDTOOLS_URL, path).await
}

/// Runs the BuildTools JAR and generates a SpigotMC JAR.
///
/// # Arguments
///
/// * `java_home` - The directory of the `JAVA_HOME` environment variable.
/// * `buildtools_jar` - The path of the BuildTools JAR file.
/// * `working_dir` - The directory where BuildTools should be run.
///                   It is not recommended for this to be the same directory as the `buildtools_jar`
/// * `version` - The Minecraft version which should be used.
///
/// # Returns
///
/// This function returns the Path of the generated Spigot JAR.
pub async fn run_buildtools<P: AsRef<Path>>(java_home: P, buildtools_jar: P, working_dir: P, version: &str) -> io::Result<PathBuf> {
    if working_dir.as_ref().exists() && working_dir.as_ref().is_dir() {
        dir::clear_directory(&working_dir)
            .await
            .map_err(|e| io::Error::new(ErrorKind::Other, format!("Failed to clear directory: {}", e)))?;
    } else {
        fs::create_dir(&working_dir)
            .map_err(|e| io::Error::new(ErrorKind::Other, format!("Failed to create directory: {}", e)))?;
    }

    let java_bin = java_home.as_ref().join(Path::new("bin/java"));
    let buildtools_jar_path = buildtools_jar.as_ref().to_str().ok_or_else(|| {
        io::Error::new(ErrorKind::InvalidInput, "Invalid BuildTools JAR path")
    })?;

    let mut command = Command::new(java_bin);
    command.stdout(io::stdout());
    command.stderr(io::stderr());
    command.current_dir(&working_dir);
    command.args(["-jar", buildtools_jar_path, "--rev", version]);

    let mut process = command.spawn()?;
    let exit_status = process.wait()?;

    if exit_status.success() {
        let file_regex = Regex::new(SPIGOT_JAR_REGEX).unwrap();
        find_file(&file_regex, &working_dir).await
    } else {
        let error_code = exit_status.code().unwrap_or(-1);
        Err(io::Error::new(
            ErrorKind::Other,
            format!("Failed to run BuildTools with exit code {}", error_code),
        ))
    }
}

/// Finds a file by a certain regex in a certain directory
///
/// # Arguments
///
/// * `regex` - The regex to which the file needs to match
/// * `directory` - The directory in which to search for
///
/// # Returns
///
/// The first file that matches the given regex
pub async fn find_file<P: AsRef<Path>>(regex: &Regex, directory: P) -> io::Result<PathBuf> {
    for file in fs::read_dir(directory.as_ref())? {
        let unwrapped_file = file?;
        if regex.is_match(unwrapped_file.file_name().to_str().unwrap()) {
            return Ok(unwrapped_file.path())
        }
    }
    Err(io::Error::new(
        ErrorKind::Other,
        "Cannot find the file",
    ))
}

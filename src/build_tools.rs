//! Downloads and runs BuildTools.

use crate::download_url;
use std::path::{Path, PathBuf};

/// The URL of the latest BuildTools JAR build from SpigotMC's Jenkins.
const BUILDTOOLS_URL: &str = "https://hub.spigotmc.org/jenkins/job/BuildTools/lastSuccessfulBuild/artifact/target/BuildTools.jar";

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
/// * `version` - The Minecraft version which should be used.
///
/// # Returns
///
/// This function returns the Path of the generated Spigot JAR.
pub fn run_buildtools<P: AsRef<Path>>(java_home: P, buildtools_jar: P, working_dir: P, version: &str) -> PathBuf {
    // temporary so compiler doesn't complain
    PathBuf::new()
}
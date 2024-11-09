//! Downloads and runs BuildTools.

use crate::USER_AGENT;
use futures_util::StreamExt;
use std::fs::File;
use std::io::Write;
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
pub async fn download_buildtools<P: AsRef<Path>>(path: P) {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .unwrap();
    let mut response_stream = client
        .get(BUILDTOOLS_URL)
        .send()
        .await
        .expect("Failed to receive response")
        .bytes_stream();

    let mut buildtools_file = File::create(path).expect("Unable to create BuildTools file");

    while let Some(chunk) = response_stream.next().await {
        let chunk = chunk.expect("Failed to read bytes");
        buildtools_file.write_all(&chunk).expect("Failed to write to file");
    }
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
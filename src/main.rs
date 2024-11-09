use futures_util::StreamExt;
use regex::Regex;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::env;
use bin_patch_gen::util::dir::create_temp_dir;

const VERSIONS_URL: &str = "https://hub.spigotmc.org/versions";
const VERSION_REGEX: &str = r"^1\.\d{1,2}(?:\.\d{1,2})?$";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:131.0) Gecko/20100101 Firefox/131.0";
const BUILDTOOLS_URL: &str = "https://hub.spigotmc.org/jenkins/job/BuildTools/lastSuccessfulBuild/artifact/target/BuildTools.jar";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let java_home = &*env::var("JAVA_HOME").expect("No JAVA_HOME set! Please set it manually if it wasn't set before.");

    println!("Fetching versions...");
    let versions = filter_versions(fetch_url(VERSIONS_URL).await);

    println!("Downloading BuildTools...");

    let temp_dir = create_temp_dir("bin-patch-gen")?;

    let buildtools_path = temp_dir.join("BuildTools.jar");
    download_buildtools(buildtools_path.to_str().unwrap()).await;

    // this is temporary, for testing
    run_buildtools(java_home, buildtools_path.to_str().unwrap(), "1.12.2", "1.12.2");

    Ok(())
}

async fn fetch_url(url: &str) -> Html {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Unable to create client");
    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to receive response")
        .text()
        .await
        .expect("Failed to parse response as string");

    Html::parse_document(&*response)
}

fn filter_versions(document: Html) -> Vec<String> {
    let version_regex = Regex::new(VERSION_REGEX).unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let mut list: Vec<String> = Vec::new();
    for element in document.select(&a_selector) {
        if let Some(ref_href) = element.value().attr("href") {
            let href = ref_href.strip_suffix(".json").unwrap_or_else(|| ref_href);

            if version_regex.is_match(href) {
                list.push(href.to_string());
            }
        }
    }

    list
}

async fn download_buildtools(path: &str) {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Unable to create client");
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

/// Runs BuildTools and generates a Spigot jar
/// returns the jar location
fn run_buildtools(java_home: &str, buildtools_jar: &str, working_dir: &str, version: &str) -> PathBuf {
    // temporary so compiler doesn't complain
    PathBuf::new()
}
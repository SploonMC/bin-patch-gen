use crate::util::dir;
use futures_util::StreamExt;
use reqwest::IntoUrl;
use scraper::Html;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{fs, io};
use bzip2::Compression;
use bzip2::write::BzEncoder;

pub mod util;
pub mod build_tools;
pub mod version;
pub mod jar;

/// The user agent being used for all HTTP requests.
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:131.0) Gecko/20100101 Firefox/131.0";

pub const JAR_VERSIONS_PATH: &str = "META-INF/versions/";

pub const MINECRAFT_VERSION_REGEX: &str = r"(1\.\d{1,2}(?:\.\d{1,2})?)";

pub const SERVER_JAR_REGEX: &str = r"server-(1\.\d{1,2}(?:\.\d{1,2})?)\.jar";

pub const SPIGOT_SERVER_JAR_REGEX: &str = r"spigot-(1\.\d{1,2}(?:\.\d{1,2})?)-R0.1-SNAPSHOT\.jar";

pub type Reqwsult<T> = Result<T, reqwest::Error>;

pub async fn get_url<U: IntoUrl>(url: U) -> Reqwsult<String> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()?;

    client
        .get(url)
        .send()
        .await
        .expect("Failed to receive response")
        .text()
        .await
}

pub async fn download_url<U: IntoUrl, P: AsRef<Path>>(url: U, path: P) -> Reqwsult<()> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()?;

    let mut stream = client
        .get(url)
        .send()
        .await
        .expect("Failed to receive response")
        .bytes_stream();

    let mut buildtools_file = File::create(path).expect("Unable to create BuildTools file");

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.expect("Failed to read bytes");
        buildtools_file.write_all(&chunk).expect("Failed to write to file");
    }

    Ok(())
}

pub async fn prepare_extraction_path(extraction_path: &Path) -> std::io::Result<()> {
    if !extraction_path.exists() || !extraction_path.is_dir() {
        fs::create_dir_all(extraction_path)?;
    } else {
        dir::clear_directory(extraction_path).await?;
    }
    Ok(())
}

/// Fetches a URL and returns the HTML.
///
/// # Arguments
///
/// * `url` - The URL.
///
/// # Returns
///
/// The site's HTML.
pub async fn fetch_url<U: IntoUrl>(url: U) -> Reqwsult<Html> {
    Ok(Html::parse_document(&*(get_url(url).await?)))
}

pub fn write_patch<P: AsRef<Path>>(vanilla_jar: P, spigot_jar: P, out: &mut impl Write) -> io::Result<()> {
    write_patch_internal(File::open(vanilla_jar)?, File::open(spigot_jar)?, out)
}

fn write_patch_internal<T>(mut vanilla: T, mut spigot: T, out: &mut impl Write) -> io::Result<()>
where
    T: Read + Write,
{
    let mut vanilla_bytes = Vec::new();
    vanilla.read_to_end(&mut vanilla_bytes)?;
    let mut spigot_bytes = Vec::new();
    spigot.read_to_end(&mut spigot_bytes)?;

    let mut diff = Vec::new();
    bsdiff::diff(&vanilla_bytes, &spigot_bytes, &mut diff)?;

    let mut encoder = BzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&*diff)?;

    let compressed = encoder.finish()?;

    fs::write(out, compressed)
}
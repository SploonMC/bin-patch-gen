use futures_util::StreamExt;
use reqwest::IntoUrl;
use scraper::Html;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub mod util;
pub mod build_tools;
pub mod version;

/// The user agent being used for all HTTP requests.
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:131.0) Gecko/20100101 Firefox/131.0";

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
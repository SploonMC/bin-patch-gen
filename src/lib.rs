use reqwest::IntoUrl;
use scraper::Html;

pub mod util;
pub mod build_tools;
pub mod version;

/// The user agent being used for all HTTP requests.
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:131.0) Gecko/20100101 Firefox/131.0";

/// Fetches a URL and returns the HTML.
/// 
/// # Arguments
/// 
/// * `url` - The URL.
/// 
/// # Returns
/// 
/// The site's HTML.
pub async fn fetch_url<U: IntoUrl>(url: U) -> Html {
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
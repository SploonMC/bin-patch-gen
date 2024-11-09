//! Fetches Minecraft versions from SpigotMC.

use regex::Regex;
use scraper::{Html, Selector};
use crate::fetch_url;

/// The URL which should be used for fetching SpigotMC versions.
const VERSIONS_URL: &str = "https://hub.spigotmc.org/versions";

/// The RegEx which should be applied to each JSON file found on the [`VERSIONS_URL`]
const VERSION_REGEX: &str = r"^1\.\d{1,2}(?:\.\d{1,2})?$";

/// Function to filter out versions from an HTML page.
/// 
/// # Arguments
/// * `document` - The HTML page.
/// 
/// # Returns
/// 
/// The filtered list of versions, based off the [`VERSION_REGEX`] and JSON files.
pub fn filter_versions(document: Html) -> Vec<String> {
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

/// Helper function for fetching all SpigotMC versions.
/// 
/// # Returns
/// All SpigotMC versions.
pub async fn fetch_versions() -> Vec<String> {
    filter_versions(fetch_url(VERSIONS_URL).await)
}
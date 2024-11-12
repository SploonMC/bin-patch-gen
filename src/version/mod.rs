//! Fetches Minecraft versions from SpigotMC.

mod schema;

use crate::version::schema::{Version, VersionMeta, VersionsResponse};
use crate::{download_url, fetch_url, get_url, Reqwsult};
use regex::Regex;
use scraper::{Html, Selector};
use std::path::Path;
use tracing::warn;

/// The URL which should be used for fetching SpigotMC versions.
const VERSIONS_URL: &str = "https://hub.spigotmc.org/versions";

/// The RegEx which should be applied to each JSON file found on the [`VERSIONS_URL`]
const VERSION_REGEX: &str = r"^1\.\d{1,2}(?:\.\d{1,2})?$";

/// The URL which should be used for fetching Piston Metadata.
const PISTON_META_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";


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
            let href = ref_href.strip_suffix(".json").unwrap_or(ref_href);

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
    filter_versions(fetch_url(VERSIONS_URL).await.unwrap_or_else(|err| {
        panic!("failed fetching versions: {err:#?}")
    }))
}

pub async fn fetch_piston_meta() -> Reqwsult<VersionsResponse> {
    let text = get_url(PISTON_META_URL).await?;
    Ok(serde_json::from_str(&text).unwrap_or_else(|err| {
        panic!("failed deserializing piston meta: {err:#?}")
    }))
}

pub async fn fetch_version_meta(versions: VersionsResponse, version: String) -> Reqwsult<VersionMeta> {
    let version = versions
        .versions
        .iter()
        .find(|ver| ver.id == version)
        .unwrap_or_else(|| {
            warn!("failed to find version {version}, falling back to latest: {}", versions.latest.release);
            versions
                .versions
                .iter()
                .find(|ver| ver.id == versions.latest.release)
                .unwrap()
        })
        .clone();

    Ok(
        serde_json::from_str(&(get_url(version.url).await?)).unwrap_or_else(|err| {
            panic!("failed deserializing version: {err:#?}")
        })
    )
}

pub async fn download_version<P: AsRef<Path>>(version: Version, path: P) -> Reqwsult<()> {
    download_url(version.url, path).await
}

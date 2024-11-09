use proc_macros::{serial, serial_snake};

#[serial]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[serial]
pub struct Version {
    pub id: String,
    pub url: String,
    #[serde(rename = "type")]
    pub version_type: String
}

#[serial]
pub struct VersionsResponse {
    pub latest: LatestVersions,
    pub versions: Vec<Version>,
}

#[serial]
pub struct VersionDownload {
    pub url: String,
}

#[serial_snake]
pub struct VersionDownloads {
    pub server: VersionDownload,
    pub server_mappings: VersionDownload,
}

#[serial]
pub struct VersionMeta {
    pub downloads: VersionDownloads,
}

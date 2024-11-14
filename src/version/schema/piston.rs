use proc_macros::{serial, serial_snake};

#[serial]
pub struct PistonLatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[serial]
pub struct PistonVersion {
    pub id: String,
    pub url: String,
    #[serde(rename = "type")]
    pub version_type: String
}

#[serial]
pub struct PistonVersionsResponse {
    pub latest: PistonLatestVersions,
    pub versions: Vec<PistonVersion>,
}

#[serial]
pub struct PistonVersionDownload {
    pub url: String,
}

#[serial_snake]
pub struct PistonVersionDownloads {
    pub server: PistonVersionDownload,
    pub server_mappings: PistonVersionDownload,
}

#[serial]
pub struct PistonVersionMeta {
    pub downloads: PistonVersionDownloads,
}

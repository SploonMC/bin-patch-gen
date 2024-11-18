use crate::util::{dir, sha1};
use build_tools::{download_buildtools, find_file, run_buildtools, VANILLA_JAR_REGEX};
use config::{Config, PatchedVersionMeta};
use futures_util::StreamExt;
use jar::extract_jar;
use qbsdiff::{Bsdiff, Bspatch};
use regex::Regex;
use reqwest::IntoUrl;
use scraper::Html;
use std::fmt::Display;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};
use tracing::{info, warn};
use util::dir::create_temp_dir;
use version::fetch_spigot_version_meta;
use version::schema::spigot::SpigotBuildData;
use crate::maven::MavenDependency;

pub mod build_tools;
pub mod config;
pub mod jar;
#[cfg(test)]
pub mod tests;
pub mod util;
pub mod version;
pub mod maven;

/// The user agent being used for all HTTP requests.
pub const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:131.0) Gecko/20100101 Firefox/131.0";

pub const JAR_VERSIONS_PATH: &str = "META-INF/versions/";

pub const MINECRAFT_VERSION_REGEX: &str = r"(1\.\d{1,2}(?:\.\d{1,2})?)";

pub const SERVER_JAR_REGEX: &str = r"server-(1\.\d{1,2}(?:\.\d{1,2})?)\.jar";

pub const SPIGOT_SERVER_JAR_REGEX: &str = r"spigot-(1\.\d{1,2}(?:\.\d{1,2})?)-R0.1-SNAPSHOT\.jar";

pub const PISTON_DATA_BASE_URL: &str = "https://piston-data.mojang.com/v1/objects";

pub type Reqwsult<T> = Result<T, reqwest::Error>;

pub async fn get_url<U: IntoUrl>(url: U) -> Reqwsult<String> {
    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;

    client
        .get(url)
        .send()
        .await
        .expect("Failed to receive response")
        .text()
        .await
}

pub async fn download_url<U: IntoUrl, P: AsRef<Path>>(url: U, path: P) -> Reqwsult<()> {
    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;

    let mut stream = client
        .get(url)
        .send()
        .await
        .expect("Failed to receive response")
        .bytes_stream();

    let mut buildtools_file = File::create(path).expect("Unable to create BuildTools file");

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.expect("Failed to read bytes");
        buildtools_file
            .write_all(&chunk)
            .expect("Failed to write to file");
    }

    Ok(())
}

pub async fn prepare_extraction_path(extraction_path: &Path) -> io::Result<()> {
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
    Ok(Html::parse_document(&(get_url(url).await?)))
}

pub fn write_patch<P, P1>(vanilla_jar: P, spigot_jar: P, out: P1) -> io::Result<()>
where
    P: AsRef<Path>,
    P1: AsRef<Path>,
{
    let mut vanilla = File::open(vanilla_jar)?;
    let mut vanilla_bytes = Vec::new();
    vanilla.read_to_end(&mut vanilla_bytes)?;

    let mut spigot = File::open(spigot_jar)?;
    let mut spigot_bytes = Vec::new();
    spigot.read_to_end(&mut spigot_bytes)?;

    let mut diff = Vec::new();

    Bsdiff::new(&vanilla_bytes, &spigot_bytes)
        .compression_level(9)
        .compare(Cursor::new(&mut diff))?;

    fs::write(out, diff)
}

pub struct MinecraftVersion(u8, u8, u8);

impl Display for MinecraftVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

impl MinecraftVersion {
    pub fn of(string: String) -> Self {
        let numbers = string
            .split(".")
            .map(|string| string.parse::<u8>().unwrap())
            .collect::<Vec<u8>>();

        Self(numbers[0], numbers[1], *numbers.get(2).unwrap_or(&0u8))
    }

    pub fn get_java_version(&self) -> u8 {
        if self.1 < 17 {
            return 8;
        }

        if self.1 == 17 {
            return 16;
        }

        if self.1 < 20 && self.2 < 5 {
            return 17;
        }

        21
    }
}

pub async fn run(
    versions: Vec<String>,
    run_dir: PathBuf,
    force_build: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let using_env = [8, 16, 17, 21]
        .iter()
        .all(|ver| std::env::var(format!("JAVA_HOME_{ver}")).is_ok());

    if using_env {
        info!("Using environment variables for java home instead of configuration file.");
    }

    let config_file = run_dir.parent().unwrap().join("config.toml");
    if !fs::exists(&config_file)? {
        fs::write(config_file, toml::to_string_pretty(&Config::default())?)?;
        info!("Generated default config file.")
    }

    let config = config::read_config("config.toml")?;

    info!("Releases found: {versions:?}");

    info!("Downloading BuildTools...");
    let temp_dir = create_temp_dir("bin-patch-gen")?;

    let buildtools_path = temp_dir.join("BuildTools.jar");
    download_buildtools(buildtools_path.clone()).await?;
    info!("Downloaded BuildTools successfully!");

    let vanilla_jar_regex = Regex::new(VANILLA_JAR_REGEX)?;
    let spigot_jar_regex = Regex::new(SPIGOT_SERVER_JAR_REGEX)?;

    if !run_dir.exists() {
        fs::create_dir_all(&run_dir)?;
    }

    for version in versions {
        info!("Building Spigot for version {}...", version);
        let version_path = temp_dir.join(Path::new(&*version));
        let work_path = version_path.join(Path::new("work"));
        let vanilla_jar_regex = vanilla_jar_regex.clone();
        let spigot_jar_regex = spigot_jar_regex.clone();
        let library_file = &run_dir.join(format!("{version}.libs"));

        let mc_version = MinecraftVersion::of(version.clone());
        let java_home = if !using_env {
            config.java_home(mc_version.get_java_version())
        } else {
            PathBuf::from(
                std::env::var(format!("JAVA_HOME_{}", mc_version.get_java_version())).unwrap(),
            )
        };

        let remote_meta = fetch_spigot_version_meta(version.clone()).await?;

        let version_file = &run_dir.join(format!("{version}.json"));
        if version_file.exists() {
            let patched_meta = PatchedVersionMeta::read(version_file);

            if patched_meta.is_err() {
                warn!("{version} metadata is invalid or could not be read! Rebuilding...");
            } else {
                let patched_meta = patched_meta.unwrap();
                if remote_meta.refs == patched_meta.commit_hashes && !force_build && library_file.exists() {
                    info!("Already built version {version}, skipping");
                    continue;
                }
            }
        }

        let result = run_buildtools(
            java_home,
            buildtools_path.clone(),
            version_path.clone(),
            &version,
        )
        .await?;
        let vanilla_jar = find_file(&vanilla_jar_regex, work_path).await?;

        info!(
            "BuildTools finished building Spigot for version {}!",
            version
        );
        info!("Built jar location: {}", result.to_str().unwrap());

        info!("Checking whether jars need extraction...");
        let needs_extraction_vanilla = jar::has_dir(&vanilla_jar, JAR_VERSIONS_PATH).unwrap();
        let needs_extraction_spigot = jar::has_dir(&result, JAR_VERSIONS_PATH).unwrap();

        let vanilla_jar = if needs_extraction_vanilla {
            info!("Vanilla jar needs extraction");
            info!("Extracting vanilla jar...");
            let vanilla_jar_extraction_path = version_path.join(Path::new("vanilla_jar"));
            prepare_extraction_path(&vanilla_jar_extraction_path)
                .await
                .unwrap();
            extract_jar(&vanilla_jar, &vanilla_jar_extraction_path).unwrap();
            info!("Successfully extracted vanilla jar!");

            let extraction_path = vanilla_jar_extraction_path.join(Path::new(JAR_VERSIONS_PATH));
            let versions_file_path = vanilla_jar_extraction_path
                .join("META-INF")
                .join("versions.list");
            let file_content = fs::read_to_string(&versions_file_path);

            if file_content.is_err() {
                warn!("Failed to read versions.list. Will use {vanilla_jar:?} instead.");
                vanilla_jar
            } else {
                let file_content = file_content.unwrap();
                let split_content = file_content.split("\t").collect::<Vec<&str>>();

                let jar_path_relative = split_content.get(2).unwrap();

                extraction_path.join(jar_path_relative)
            }
        } else {
            info!("Vanilla jar does not need extraction");
            vanilla_jar
        };

        let spigot_jar = if needs_extraction_spigot {
            info!("Spigot jar needs extraction");
            info!("Extracting spigot jar...");
            let spigot_jar_extraction_path = version_path.join(Path::new("spigot_jar"));
            prepare_extraction_path(&spigot_jar_extraction_path)
                .await
                .unwrap();
            extract_jar(&result, &spigot_jar_extraction_path).unwrap();
            info!("Successfully extracted spigot jar!");

            let file = find_file(
                &spigot_jar_regex,
                spigot_jar_extraction_path.join(Path::new(JAR_VERSIONS_PATH)),
            )
            .await;
            if let Ok(file) = file {
                file
            } else {
                warn!("Failed to read versions.list. Will use {result:?} instead.");
                result
            }
        } else {
            info!("Spigot jar does not need extraction");
            result
        };

        let patch_file = &run_dir.join(format!("{version}.patch"));

        info!("Generating diff...");
        write_patch(&vanilla_jar, &spigot_jar, patch_file)?;
        info!("Diff generated!");

        let vanilla_jar_hash = sha1(vanilla_jar)?;
        
        info!("Reading BuildData...");
        let build_data_info = version_path.join("BuildData/info.json");
        let fallback_vanilla_download_url = format!("{PISTON_DATA_BASE_URL}/{vanilla_jar_hash}/server.jar");
        
        let vanilla_download_url = if build_data_info.exists() {
            match serde_json::from_str::<SpigotBuildData>(&fs::read_to_string(build_data_info)?) {
                Ok(data) => data.server_url,
                Err(_) => fallback_vanilla_download_url
            }
        } else {
            fallback_vanilla_download_url
        };
        info!("Read BuildData!");

        let patched_meta = PatchedVersionMeta {
            patch_file: patch_file
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            commit_hashes: remote_meta.refs,
            patch_hash: sha1(patch_file)?,
            vanilla_jar_hash,
            patched_jar_hash: sha1(spigot_jar)?,
            vanilla_download_url,
        };

        patched_meta.write(version_file)?;
        info!("Wrote version metadata file!");
        
        let server_pom = version_path.join("Spigot/Spigot-Server/pom.xml");
        
        let (project, maven_dependencies) = maven::read_dependencies(server_pom)?;
        
        MavenDependency::write(project, library_file, maven_dependencies)?;
    }

    Ok(())
}

pub async fn patch<P: AsRef<Path>>(old: P, new: P, patch: P) -> io::Result<()> {
    let mut patch_file = File::open(patch)?;
    let mut patch_buf = vec![];
    patch_file.read_to_end(&mut patch_buf)?;

    info!("Patching...");

    let mut old_file = File::open(old)?;
    let mut old_buf = vec![];
    old_file.read_to_end(&mut old_buf)?;

    let mut new_file = File::create(new)?;
    let mut new_buf = vec![];

    let patcher = Bspatch::new(&patch_buf)?;
    patcher.apply(&old_buf, &mut new_buf)?;

    new_file.write_all(&new_buf)?;

    info!("Patched!");

    Ok(())
}

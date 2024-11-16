use bin_patch_gen::build_tools::{
    download_buildtools, find_file, run_buildtools, VANILLA_JAR_REGEX,
};
use bin_patch_gen::config::{Config, PatchedVersionMeta};
use bin_patch_gen::jar::extract_jar;
use bin_patch_gen::util::dir::create_temp_dir;
use bin_patch_gen::util::{sha1, TimeFormatter};
use bin_patch_gen::version::{fetch_spigot_version_meta, fetch_versions};
use bin_patch_gen::{config, jar, prepare_extraction_path, write_patch, MinecraftVersion, JAR_VERSIONS_PATH, MINECRAFT_VERSION_REGEX, SPIGOT_SERVER_JAR_REGEX};
use clap::{command, Parser, Subcommand};
use qbsdiff::Bspatch;
use regex::Regex;
use std::env::current_dir;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use tracing_subscriber::fmt::format;

#[derive(Parser)]
#[command(about, long_about = None)]
/// The binary patch generator for Sploon.
struct Cli {
    /// The version to generate for. If not specified, it will generate patches for
    /// all versions of Spigot.
    #[arg(short, long, value_name = "version")]
    pub version: Option<String>,

    /// Whether we should clean the run directory.
    #[arg(short, long, value_name = "clean")]
    pub clean: bool,

    #[arg(short = 'f', long = "force", value_name = "force")]
    pub force_build: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Patches a file.
    Patch {
        /// The old file.
        old: PathBuf,
        /// The new file, patched.
        new: PathBuf,
        /// The bsdiff patch file, compressed with bzip2.
        patch: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fmt = format()
        .with_file(true)
        .with_line_number(true)
        .with_timer(TimeFormatter);

    tracing_subscriber::fmt().event_format(fmt).init();

    let cli = Cli::parse();

    if let Some(Commands::Patch { old, new, patch }) = cli.command {
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

        return Ok(());
    }

    let versions = if let Some(version) = cli.version {
        vec![version]
    } else {
        fetch_versions().await
    };

    let run_dir = if current_dir()?.ends_with("run") {
        current_dir()?
    } else {
        current_dir()?.join("run")
    };

    if cli.clean {
        info!("Cleaning run directory");
        fs::remove_dir_all(&run_dir)?;
        fs::create_dir_all(&run_dir)?;
    }

    run(versions, run_dir, cli.force_build).await
}

async fn run(versions: Vec<String>, run_dir: PathBuf, force_build: bool) -> Result<(), Box<dyn std::error::Error>> {
    let using_env = [8, 16, 17, 21]
        .iter()
        .all(|ver| std::env::var(format!("JAVA_HOME_{ver}")).is_ok());

    if using_env {
        info!("Using environment variables for java home instead of configuration file.");
    }

    let config_file = PathBuf::from("config.toml");
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
    let minecraft_version_regex = Regex::new(MINECRAFT_VERSION_REGEX)?;
    let spigot_jar_regex = Regex::new(SPIGOT_SERVER_JAR_REGEX)?;

    if !run_dir.exists() {
        fs::create_dir_all(&run_dir)?;
    }

    for version in versions {
        info!("Building Spigot for version {}...", version);
        let version_path = temp_dir.join(Path::new(&*version));
        let work_path = version_path.join(Path::new("work"));
        let vanilla_jar_regex = vanilla_jar_regex.clone();
        let minecraft_version_regex = minecraft_version_regex.clone();
        let spigot_jar_regex = spigot_jar_regex.clone();

        let mc_version = MinecraftVersion::of(version.clone());
        let java_home = if !using_env {
            config.java_home(mc_version.get_java_version())
        } else {
            PathBuf::from(std::env::var(format!("JAVA_HOME_{}", mc_version.get_java_version())).unwrap())
        };

        let remote_meta = fetch_spigot_version_meta(version.clone()).await?;

        let version_file = &run_dir.join(format!("{version}.json"));
        if version_file.exists() {
            let patched_meta = PatchedVersionMeta::read(version_file);

            if patched_meta.is_err() {
                warn!("{version} metadata is invalid or could not be read! Rebuilding...");
            } else {
                let patched_meta = patched_meta.unwrap();
                if remote_meta.refs == patched_meta.commit_hashes && !force_build {
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
            let versions_file_path = vanilla_jar_extraction_path.join("META-INF").join("versions.list");
            let file_content = fs::read_to_string(&versions_file_path);

            if file_content.is_err() {
                warn!("Failed to read versions.list. Will use {vanilla_jar:?} instead.");
                vanilla_jar
            } else {
                let file_content = file_content.unwrap();
                let split_content = file_content
                    .split("    ")
                    .collect::<Vec<&str>>();

                let jar_path_relative = split_content
                    .get(2)
                    .unwrap();

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
            if file.is_err() {
                warn!("Failed to read versions.list. Will use {result:?} instead.");
                result
            } else {
                file.unwrap()
            }
        } else {
            info!("Spigot jar does not need extraction");
            result
        };

        let patch_file = &run_dir.join(format!("{version}.patch"));

        info!("Generating diff...");
        write_patch(&vanilla_jar, &spigot_jar, patch_file)?;
        info!("Diff generated!");

        let patched_meta = PatchedVersionMeta {
            patch_file: patch_file
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            commit_hashes: remote_meta.refs,
            patch_hash: sha1(patch_file)?,
            vanilla_jar_hash: sha1(vanilla_jar)?,
            patched_jar_hash: sha1(spigot_jar)?,
        };

        patched_meta.write(version_file)?;
        info!("Wrote version metadata file!");
    }

    Ok(())
}

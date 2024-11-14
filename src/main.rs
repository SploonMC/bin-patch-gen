use bin_patch_gen::build_tools::{
    download_buildtools, find_file, run_buildtools, VANILLA_JAR_REGEX,
};
use bin_patch_gen::config::{Config, PatchedVersionMeta};
use bin_patch_gen::jar::extract_jar;
use bin_patch_gen::util::dir::create_temp_dir;
use bin_patch_gen::util::{sha1, TimeFormatter};
use bin_patch_gen::version::{fetch_spigot_version_meta, fetch_versions};
use bin_patch_gen::{
    config, jar, prepare_extraction_path, write_patch, MinecraftVersion, JAR_VERSIONS_PATH,
};
use bzip2::read::BzDecoder;
use clap::{command, Parser, Subcommand};
use regex::Regex;
use std::env::current_dir;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::info;
use tracing_subscriber::fmt::format;

#[derive(Parser)]
#[command(about, long_about = None)]
/// The binary patch generator for Sploon.
struct Cli {
    /// The version to generate for. If not specified, it will generate patches for
    /// all versions of Spigot.
    #[arg(short, long, value_name = "version")]
    pub version: Option<String>,

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
        let patch_file = File::open(patch)?;
        let mut old_file = File::open(old)?;
        let mut new_file = File::create(new)?;

        let mut old_buf = vec![];
        let mut new_buf = vec![];

        old_file.read_to_end(&mut old_buf)?;

        let mut decompressor = BzDecoder::new(patch_file);

        info!("Patching...");
        bsdiff::patch(&old_buf, &mut decompressor, &mut new_buf)?;

        new_file.write_all(&new_buf)?;
        info!("Patched!");

        return Ok(());
    }

    let versions = if let Some(version) = cli.version {
        vec![version]
    } else {
        fetch_versions().await
    };

    run(versions).await
}

async fn run(versions: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
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

    let run_dir = if current_dir()?.ends_with("run") {
        current_dir()?
    } else {
        current_dir()?.join("run")
    };

    if !run_dir.exists() {
        fs::create_dir_all(&run_dir)?;
    }

    for version in versions {
        info!("Building Spigot for version {}...", version);
        let version_path = temp_dir.join(Path::new(&*version));
        let work_path = version_path.join(Path::new("work"));
        let regex = vanilla_jar_regex.clone();

        let mc_version = MinecraftVersion::of(version.clone());
        let java_home = config.java_home(mc_version.get_java_version());
        let remote_meta = fetch_spigot_version_meta(version.clone()).await?;

        let version_file = &run_dir.join(format!("{version}.json"));
        if version_file.exists() {
            let patched_meta = PatchedVersionMeta::read(version_file)?;

            if remote_meta.refs == patched_meta.commit_hashes {
                info!("Already built version {version}, skipping");
                continue;
            }
        }

        let result = run_buildtools(
            java_home,
            buildtools_path.clone(),
            version_path.clone(),
            &version,
        )
        .await?;
        let vanilla_jar = find_file(&regex, work_path).await?;

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

            let versions_folder = find_file(
                &vanilla_jar_regex,
                vanilla_jar_extraction_path.join(Path::new(JAR_VERSIONS_PATH)),
            )
            .await
            .unwrap();
            find_file(&regex, versions_folder).await.unwrap()
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

            find_file(
                &regex,
                spigot_jar_extraction_path.join(Path::new(JAR_VERSIONS_PATH)),
            )
            .await
            .unwrap()
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
            patched_jar_hash: sha1(spigot_jar)?,
        };

        patched_meta.write(version_file)?;
        info!("Wrote version metadata file!");
    }

    Ok(())
}

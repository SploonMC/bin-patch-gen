use crate::{
    config::PatchedVersionMeta,
    download_url,
    jar::{self, extract_jar},
    prepare_extraction_path, run,
    util::{sha1, TimeFormatter},
    JAR_VERSIONS_PATH,
};
use std::{
    env::current_dir,
    fs,
    io::Result,
    path::{Path, PathBuf},
};
use tokio::test;
use tracing::{info, warn};
use tracing_subscriber::fmt::format;

fn run_dir() -> Result<PathBuf> {
    let run_dir = if current_dir()?.ends_with("run") {
        current_dir()?.join("tests")
    } else {
        current_dir()?.join("run/tests")
    };

    Ok(run_dir)
}

async fn test_version(version: String) -> Result<()> {
    println!();

    let fmt = format()
        .with_file(true)
        .with_line_number(true)
        .with_timer(TimeFormatter);
    let _ = tracing_subscriber::fmt().event_format(fmt).try_init();

    let run_dir = run_dir()
        .expect("failed retrieving run directory")
        .join(&version);

    run(vec![version.clone()], run_dir.clone(), true)
        .await
        .expect("failed running patch gen");

    let patched_meta = PatchedVersionMeta::read(run_dir.join(format!("{version}.json")))
        .expect("failed reading patched meta");
    let patch = &run_dir.join(format!("{version}.patch"));
    let vanilla_jar_path = &run_dir.join(format!("{version}-vanilla.jar"));
    let spigot_jar_path = &run_dir.join(format!("{version}-patched.jar"));

    info!("Files downloaded");

    download_url(patched_meta.vanilla_download_url, vanilla_jar_path)
        .await
        .expect("failed downloading vanilla");

    info!("Checking whether vanilla jar needs extraction");
    let vanilla_jar_needs_extraction =
        jar::has_dir(vanilla_jar_path, JAR_VERSIONS_PATH).expect("failed reading vanilla jarfile");

    let vanilla_jar_path = if vanilla_jar_needs_extraction {
        info!("Vanilla jar needs extraction");
        info!("Extracting vanilla jar...");

        let vanilla_jar_extraction_path = &run_dir.join(Path::new("vanilla_jar"));
        prepare_extraction_path(vanilla_jar_extraction_path)
            .await
            .expect("failed preparing extraction path");
        extract_jar(&vanilla_jar_path, &vanilla_jar_extraction_path)
            .expect("failed extracting jar");
        info!("Successfully extracted vanilla jar!");

        let extraction_path = vanilla_jar_extraction_path.join(Path::new(JAR_VERSIONS_PATH));
        let versions_file_path = vanilla_jar_extraction_path
            .join("META-INF")
            .join("versions.list");
        let file_content = fs::read_to_string(&versions_file_path);

        if file_content.is_err() {
            warn!("Failed to read versions.list. Will use {vanilla_jar_path:?} instead.");
            vanilla_jar_path
        } else {
            let file_content = file_content.unwrap();
            let split_content = file_content.split("\t").collect::<Vec<&str>>();

            let jar_path_relative = split_content.get(2).unwrap();

            &extraction_path.join(jar_path_relative)
        }
    } else {
        info!("Vanilla jar does not need extraction.");
        vanilla_jar_path
    };

    assert_eq!(
        sha1(vanilla_jar_path).expect("failed hashing vanilla jar"),
        patched_meta.vanilla_jar_hash
    );
    assert_eq!(
        sha1(patch).expect("failed hashing patch"),
        patched_meta.patch_hash
    );

    crate::patch(vanilla_jar_path, spigot_jar_path, patch)
        .await
        .expect("failed patching");

    assert_eq!(
        sha1(spigot_jar_path).expect("failed hashing spigot jar"),
        patched_meta.patched_jar_hash
    );

    Ok(())
}

macro_rules! tests {
    ($($version:literal),+) => {
        $(
            paste::paste! {
                #[test]
                async fn [<test_ $version>]() {
                    test_version(stringify!($version).to_owned().replace("_", ".")).await.unwrap();
                }
            }
        )+
    };
}

tests!(1_21_3, 1_10, 1_17, 1_8);

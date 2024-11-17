use crate::{
    config::PatchedVersionMeta,
    download_url, run,
    util::{sha1, TimeFormatter},
};
use std::{env::current_dir, io::Result, path::PathBuf};
use tokio::test;
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

    let run_dir = run_dir().expect("failed retrieving run directory");

    run(vec![version.clone()], run_dir.clone(), true)
        .await
        .expect("failed running patch gen");

    let patched_meta = PatchedVersionMeta::read(&run_dir.join(format!("{version}.json")))
        .expect("failed reading patched meta");
    let patch = &run_dir.join(format!("{version}.patch"));
    let vanilla_jar_path = &run_dir.join(format!("{version}-vanilla.jar"));
    let spigot_jar_path = &run_dir.join(format!("{version}-patched.jar"));

    download_url(patched_meta.vanilla_download_url, vanilla_jar_path)
        .await
        .expect("failed downloading vanilla");

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

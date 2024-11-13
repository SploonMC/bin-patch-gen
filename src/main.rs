use bin_patch_gen::build_tools::{
    download_buildtools, find_file, run_buildtools, VANILLA_JAR_REGEX,
};
use bin_patch_gen::config::Config;
use bin_patch_gen::jar::extract_jar;
use bin_patch_gen::util::dir::create_temp_dir;
use bin_patch_gen::util::TimeFormatter;
use bin_patch_gen::version::fetch_versions;
use bin_patch_gen::{config, jar, prepare_extraction_path, write_patch, MinecraftVersion, JAR_VERSIONS_PATH};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;
use tracing_subscriber::fmt::format;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fmt = format()
        .with_file(true)
        .with_line_number(true)
        .with_timer(TimeFormatter);

    tracing_subscriber::fmt().event_format(fmt).init();

    let config_file = PathBuf::from("config.toml");
    if !fs::exists(&config_file)? {
        fs::write(config_file, toml::to_string_pretty(&Config::default())?)?;
        info!("Generated default config file.")
    }

    let config = config::read_config("config.toml")?;

    info!("Fetching versions...");
    let versions = fetch_versions().await;

    info!("Releases found: {versions:?}");

    info!("Downloading BuildTools...");
    let temp_dir = create_temp_dir("bin-patch-gen")?;

    let buildtools_path = temp_dir.join("BuildTools.jar");
    download_buildtools(buildtools_path.clone()).await?;
    info!("Downloaded BuildTools successfully!");

    let vanilla_jar_regex = Regex::new(VANILLA_JAR_REGEX)?;

    for version in versions {
        info!("Building Spigot for version {}...", version);
        let version_path = temp_dir.join(Path::new(&*version));
        let work_path = version_path.join(Path::new("work"));
        let regex = vanilla_jar_regex.clone();

        let mc_version = MinecraftVersion::of(version.clone());
        let java_home = config.java_home(mc_version.get_java_version());

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

        info!("Generating diff...");
        write_patch(vanilla_jar, spigot_jar, "bsdiff.patch")?;
        info!("Diff generated!");
    }

    Ok(())
}

use bin_patch_gen::build_tools::{download_buildtools, find_file, run_buildtools, VANILLA_JAR_REGEX};
use bin_patch_gen::util::dir::create_temp_dir;
use bin_patch_gen::version::fetch_versions;
use std::env;
use std::path::Path;
use regex::Regex;
use tracing::info;
use tracing_subscriber::fmt::format;
use bin_patch_gen::jar::extract_jar;
use bin_patch_gen::{jar, prepare_extraction_path, JAR_VERSIONS_PATH, MINECRAFT_VERSION_REGEX, SERVER_JAR_REGEX, SPIGOT_SERVER_JAR_REGEX};
use bin_patch_gen::util::TimeFormatter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fmt = format()
        .with_file(true)
        .with_line_number(true)
        .with_timer(TimeFormatter);

    tracing_subscriber::fmt()
        .event_format(fmt)
        .init();

    let java_home = &*env::var("JAVA_HOME").expect("No JAVA_HOME set! Please set it manually if it wasn't set before.");

    info!("Fetching versions...");
    let versions = fetch_versions().await;
    
    info!("Releases found: {versions:?}");

    info!("Downloading BuildTools...");
    let temp_dir = create_temp_dir("bin-patch-gen")?;

    let buildtools_path = temp_dir.join("BuildTools.jar");
    download_buildtools(buildtools_path.clone()).await?;
    info!("Downloaded BuildTools successfully!");

    for version in versions {
        info!("Building Spigot for version {}...", version);
        let version_path = temp_dir.join(Path::new(&*version));
        let work_path = version_path.join(Path::new("work"));

        let result = run_buildtools(
            java_home,
            buildtools_path.to_str().unwrap(),
            version_path.to_str().unwrap(),
            &*version
        ).await?;
        let vanilla_jar = find_file(Regex::new(VANILLA_JAR_REGEX).unwrap(), work_path).await?;

        info!("BuildTools finished building Spigot for version {}!", version);
        info!("Built jar location: {}", result.to_str().unwrap());


        info!("Checking whether jars need extraction...");
        let needs_extraction_vanilla = jar::has_dir(&vanilla_jar, JAR_VERSIONS_PATH).unwrap();
        let needs_extraction_spigot = jar::has_dir(&result, JAR_VERSIONS_PATH).unwrap();

        let vanilla_jar = if needs_extraction_vanilla {
            info!("Extracting vanilla jar...");
            let vanilla_jar_extraction_path = version_path.join(Path::new("vanilla_jar"));
            prepare_extraction_path(&vanilla_jar_extraction_path).await.unwrap();
            extract_jar(&vanilla_jar, &vanilla_jar_extraction_path).unwrap();
            info!("Successfully extracted vanilla jar!");

            let versions_folder = find_file(
                Regex::new(MINECRAFT_VERSION_REGEX).unwrap(),
                vanilla_jar_extraction_path.join(Path::new(JAR_VERSIONS_PATH))
            ).await.unwrap();
            find_file(Regex::new(SERVER_JAR_REGEX).unwrap(), versions_folder).await.unwrap()
        } else {
            vanilla_jar
        };

        let spigot_jar = if needs_extraction_spigot {
            info!("Extracting spigot jar...");
            let spigot_jar_extraction_path = version_path.join(Path::new("spigot_jar"));
            prepare_extraction_path(&spigot_jar_extraction_path).await.unwrap();
            extract_jar(&result, &spigot_jar_extraction_path).unwrap();
            info!("Successfully extracted spigot jar!");

            find_file(
                Regex::new(SPIGOT_SERVER_JAR_REGEX).unwrap(),
                spigot_jar_extraction_path.join(Path::new(JAR_VERSIONS_PATH))
            ).await.unwrap()
        } else {
            result
        };

        // TODO make the code less messy and diff the files
    }

    Ok(())
}

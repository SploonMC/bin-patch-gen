use bin_patch_gen::build_tools::{download_buildtools, run_buildtools};
use bin_patch_gen::util::dir::create_temp_dir;
use bin_patch_gen::version::{fetch_piston_meta, fetch_version_meta};
use std::env;
use tracing::info;
use tracing_subscriber::fmt::format;
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
    let versions = fetch_piston_meta().await?;
    let version_ids = versions
        .versions
        .iter()
        .filter(|ver| ver.version_type == "release")
        .map(|ver| ver.id.clone())
        .collect::<Vec<String>>();
    
    info!("Releases found: {version_ids:?}");

    info!("Downloading BuildTools...");

    let temp_dir = create_temp_dir("bin-patch-gen")?;

    let buildtools_path = temp_dir.join("BuildTools.jar");
    download_buildtools(buildtools_path.to_str().unwrap()).await?;

    info!("Fetching meta for version 1.21.3...");
    info!("Meta for 1.21.3: \n{:#?}", fetch_version_meta(versions, "1.21.3".to_string()).await?);

    // this is temporary, for testing
    run_buildtools(java_home, buildtools_path.to_str().unwrap(), "1.12.2", "1.12.2");

    Ok(())
}

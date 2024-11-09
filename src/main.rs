use bin_patch_gen::build_tools::{download_buildtools, run_buildtools};
use bin_patch_gen::util::dir::create_temp_dir;
use bin_patch_gen::version::fetch_versions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let java_home = &*env::var("JAVA_HOME").expect("No JAVA_HOME set! Please set it manually if it wasn't set before.");

    println!("Fetching versions...");
    let versions = fetch_versions().await;

    println!("Downloading BuildTools...");

    let temp_dir = create_temp_dir("bin-patch-gen")?;

    let buildtools_path = temp_dir.join("BuildTools.jar");
    download_buildtools(buildtools_path.to_str().unwrap()).await;

    // this is temporary, for testing
    run_buildtools(java_home, buildtools_path.to_str().unwrap(), "1.12.2", "1.12.2");

    Ok(())
}

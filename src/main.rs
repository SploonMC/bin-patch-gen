use bin_patch_gen::run;
use bin_patch_gen::util::TimeFormatter;
use bin_patch_gen::version::fetch_versions;
use clap::{command, Parser, Subcommand};
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;
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
        bin_patch_gen::patch(old, new, patch).await?;

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

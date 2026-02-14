use std::path::PathBuf;

use clap::Parser;
use offline_vendoring::{DownloadSkip, PackagingError, Settings, package};
use thiserror::Error;
use tracing::debug;

#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
enum Cargo {
    OfflinePackage(Cli),
}

#[derive(Parser)]
#[clap(version, about, long_about = None, args_conflicts_with_subcommands = true)]
/// Package external resources for offline environment
struct Cli {
    /// Path to config without the file extension (supported formats: JSON, TOML, YAML, INI, RON)
    /// E.g., "config/settings" instead of "config/settings.json"
    config: PathBuf,
    /// Skip one or more downloading steps
    #[clap(long, short, value_enum)]
    skip_download: Vec<DownloadSkip>,
}

#[derive(Error, Debug)]
/// Errors exposed to the CLI user
enum CliError {
    #[error("Invalid config: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Cannot deserialize config: {0}")]
    DeserializeConfig(String),
    #[error(transparent)]
    PackagingError(#[from] Box<PackagingError>),
}

fn main() -> Result<(), CliError> {
    // Install global subscriber configured based on RUST_LOG environment variable
    tracing_subscriber::fmt::init();

    let Cargo::OfflinePackage(cli) = Cargo::parse();
    debug!("Config file: {}", cli.config.display());

    let settings = config::Config::builder()
        .add_source(config::File::with_name(&cli.config.display().to_string()).required(false))
        .build()?;
    let settings: Settings = settings
        .try_deserialize()
        .map_err(|e| CliError::DeserializeConfig(e.to_string()))?;
    debug!("Got the following settings: {settings:#?}");

    package(&settings, &cli.skip_download)?;
    Ok(())
}

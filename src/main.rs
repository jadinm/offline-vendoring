use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;
use thiserror::Error;
use tracing::debug;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to config without the file extension (supported formats: JSON, TOML, YAML, INI, RON)
    /// E.g., "config/settings" instead of "config/settings.json"
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
struct Settings;

#[derive(Error, Debug)]
/// Errors exposed to the CLI user
enum CliError {
    #[error("Invalid config: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Cannot deserialize config: {0}")]
    DeserializeConfig(String),
}

fn main() -> Result<(), CliError> {
    // Install global subscriber configured based on RUST_LOG environment variable
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    if let Some(config_path) = cli.config.as_deref() {
        debug!("Config file: {}", config_path.display());
    }

    let settings = if let Some(config_base_name) = cli.config {
        let settings = config::Config::builder()
            .add_source(
                config::File::with_name(&config_base_name.display().to_string()).required(false),
            )
            .build()?;
        settings
            .try_deserialize()
            .map_err(|e| CliError::DeserializeConfig(e.to_string()))?
    } else {
        Settings
    };

    debug!("Got the following settings: {settings:#?}");
    Ok(())
}

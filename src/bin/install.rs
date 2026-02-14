use std::path::PathBuf;

use clap::Parser;
use offline_vendoring::{InstallSkip, InstallingError, PythonConfigLevel, install};
use thiserror::Error;
use tracing::debug;

#[derive(Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
enum Cargo {
    OfflineInstall(Cli),
}

#[derive(Parser)]
#[clap(version, about, long_about = None, args_conflicts_with_subcommands = true)]
/// Install packaged external resources in offline environment
struct Cli {
    /// Path to the archive containing external resources
    archive: PathBuf,

    /// Set the configuration level of python configuration
    #[clap(long, short, value_enum, default_value = "user")]
    python_config_level: PythonConfigLevel,
    /// Limit configuration to the given path (e.g., the root of your project).
    /// By default, the user-level config is modified.
    #[clap(long, short)]
    rust_config_for: Option<PathBuf>,

    /// Skip one or more install steps
    #[clap(long, short, value_enum)]
    skip: Vec<InstallSkip>,
}

#[derive(Error, Debug)]
/// Errors exposed to the CLI user
enum CliError {
    #[error(transparent)]
    InstallingError(#[from] Box<InstallingError>),
    #[error("Invalid archive path: {0}")]
    InvalidArchivePath(String),
}

fn main() -> Result<(), CliError> {
    // Install global subscriber configured based on RUST_LOG environment variable
    tracing_subscriber::fmt::init();

    let Cargo::OfflineInstall(cli) = Cargo::parse();
    debug!("Archive file: {}", cli.archive.display());
    if !cli.archive.is_file() {
        return Err(CliError::InvalidArchivePath(
            cli.archive.display().to_string(),
        ));
    }

    install(
        cli.archive.as_path(),
        &cli.python_config_level,
        cli.rust_config_for.as_ref(),
        &cli.skip,
    )?;
    Ok(())
}

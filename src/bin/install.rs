use std::path::PathBuf;

use clap::Parser;
use offline_vendoring::{InstallingError, install};
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
}

#[derive(Error, Debug)]
/// Errors exposed to the CLI user
enum CliError {
    #[error(transparent)]
    InstallingError(#[from] InstallingError),
}

fn main() -> Result<(), CliError> {
    // Install global subscriber configured based on RUST_LOG environment variable
    tracing_subscriber::fmt::init();

    let Cargo::OfflineInstall(cli) = Cargo::parse();
    debug!("Archive file: {}", cli.archive.display());
    install(cli.archive.as_path())?;
    Ok(())
}

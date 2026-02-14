use std::{env::VarError, path::PathBuf};

use thiserror::Error;
use toml_edit::TomlError;

use crate::cmd::CommandFailedError;

#[derive(Error, Debug)]
pub enum RustError {
    #[error("Cannot archive '{src}' to '{dst}': {source}")]
    Archive {
        src: PathBuf,
        dst: String,
        source: std::io::Error,
    },
    #[error(transparent)]
    CommandFailed(#[from] Box<CommandFailedError>),
    #[error("Cannot create rust sub-directory at '{0}': {1}")]
    CreateMainDirectory(PathBuf, #[source] std::io::Error),
    #[error(transparent)]
    CargoConfig(#[from] CargoHomeError),
}

#[derive(Error, Debug)]
pub enum CargoHomeError {
    #[error("Cannot find cargo config since neither ${{CARGO_HOME}} nor ${{HOME}} are set: {0}")]
    NoCargoHome(#[from] VarError),
    #[error("Cannot read tools directory at '{0}': {1}")]
    ReadToolsDirectory(PathBuf, #[source] std::io::Error),
    #[error("Cannot copy tool from {0} to {1}: {2}")]
    ImportTool(PathBuf, PathBuf, std::io::Error),
    #[error("Invalid toml in cargo config at {0}: {1}")]
    CargoConfigRead(PathBuf, TomlError),
    #[error("Cannot write updated cargo config to {0}: {1}")]
    CargoConfigWrite(PathBuf, std::io::Error),
}

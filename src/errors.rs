use thiserror::Error;
use url::Url;

use crate::cmd::CommandFailedError;

#[derive(Error, Debug)]
pub enum PackagingError {
    #[error("Failed to create output directory: {0}")]
    DirectoryCreation(#[source] std::io::Error),
    #[error(transparent)]
    CommandFailed(#[from] CommandFailedError),
    #[error("Creating archive failed: {0}")]
    TarStart(#[source] std::io::Error),
    #[error("Append folder to archive failed: {0}")]
    TarAppend(#[source] std::io::Error),
    #[error("Finalizing archive failed: {0}")]
    TarFinish(#[source] std::io::Error),
    #[error("Invalid UTF-8 character in {0}")]
    InvalidCharacter(Url),
    #[error("Cannot get a valid output path from name: {0}")]
    InvalidOutPath(#[source] std::io::Error),
}

#[derive(Error, Debug)]
pub enum InstallingError {
    #[error("Invalid config: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Cannot deserialize config: {0}")]
    DeserializeConfig(String),
    #[error("Failed to create output directory: {0}")]
    DirectoryCreation(#[source] std::io::Error),
    #[error("Uncompress archive failed: {0}")]
    TarUncompress(#[source] std::io::Error),
    #[error(transparent)]
    CommandFailed(#[from] CommandFailedError),
    #[error("Failed to read directory: {0}")]
    ReadDirectory(#[source] std::io::Error),
    #[error("Failed to recursively read the directory: {0}")]
    WalkDirectory(#[from] walkdir::Error),
    #[error("Failed to copy file: {0}")]
    Copy(#[source] std::io::Error),
    #[error("Invalid custom install command: {0}")]
    CustomInstallCommand(String),
    #[error("Failed to find cargo home: {0}")]
    NoCargoHome(#[source] std::env::VarError),
    #[error("Failed to read cargo config: {0}")]
    CargoConfigRead(#[source] toml_edit::TomlError),
    #[error("Failed to write cargo config: {0}")]
    CargoConfigWrite(#[source] std::io::Error),
    #[error("Invalid UTF-8 character in {0}")]
    InvalidCharacter(Url),
    #[error("Cannot get a valid output path from name: {0}")]
    InvalidOutPath(#[source] std::io::Error),
}

use std::path::PathBuf;

use thiserror::Error;

use crate::{
    custom::errors::CustomError, git::errors::GitError, python::errors::PythonError,
    rust::errors::RustError,
};

#[derive(Error, Debug)]
pub enum PackagingError {
    #[error("Failed to create archive: {0}")]
    ArchiveCreation(#[source] std::io::Error),
    #[error("Failed to insert settings into the archive: {0}")]
    ArchiveInsert(#[source] std::io::Error),
    #[error("Cannot create intermediate output directory at '{0}': {1}")]
    CreateMainDirectory(PathBuf, #[source] std::io::Error),
    #[error("Cannot get the absolute path for the intermediate output directory '{0}': {1}")]
    GetCannonMainDirectory(PathBuf, #[source] std::io::Error),
    #[error("Custom tasks: {0}")]
    Custom(#[from] CustomError),
    #[error("Git: {0}")]
    Git(#[from] GitError),
    #[error("Python: {0}")]
    Python(#[from] PythonError),
    #[error("Rust: {0}")]
    Rust(#[from] RustError),
}

#[derive(Error, Debug)]
pub enum InstallingError {
    #[error("Cannot find archive file name prefix in {0}")]
    InvalidArchivePath(PathBuf),
    #[error("Cannot create output directory at '{0}': {1}")]
    CreateMainDirectory(PathBuf, #[source] std::io::Error),
    #[error("Cannot get the absolute path for the output directory '{0}': {1}")]
    GetCannonMainDirectory(PathBuf, #[source] std::io::Error),
    #[error("Invalid config: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Cannot deserialize config: {0}")]
    DeserializeConfig(String),
    #[error("Open & uncompress archive failed: {0}")]
    ArchiveUncompress(#[source] std::io::Error),
    #[error("Custom tasks: {0}")]
    Custom(#[from] CustomError),
    #[error("Git: {0}")]
    Git(#[from] GitError),
    #[error("Python: {0}")]
    Python(#[from] PythonError),
    #[error("Rust: {0}")]
    Rust(#[from] RustError),
}

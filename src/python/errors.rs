use std::path::PathBuf;

use thiserror::Error;

use crate::cmd::CommandFailedError;

#[derive(Error, Debug)]
pub enum PythonError {
    #[error("Cannot archive '{src}' to '{dst}': {source}")]
    Archive {
        src: PathBuf,
        dst: String,
        source: std::io::Error,
    },
    #[error(transparent)]
    CommandFailed(#[from] Box<CommandFailedError>),
    #[error("Cannot create python sub-directory at '{0}': {1}")]
    CreateMainDirectory(PathBuf, #[source] std::io::Error),
}

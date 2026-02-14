use std::path::PathBuf;

use thiserror::Error;

use crate::cmd::CommandFailedError;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Cannot archive '{src}' to '{dst}': {source}")]
    Archive {
        src: PathBuf,
        dst: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    CommandFailed(#[from] Box<CommandFailedError>),
    #[error("Cannot parse command '{0}' as a shell command")]
    CommandUnparsable(String),
    #[error("Cannot loop over all files of '{0}': {1}")]
    WalkDirectory(PathBuf, #[source] walkdir::Error),
}

use std::path::PathBuf;

use thiserror::Error;
use url::Url;

use crate::cmd::CommandFailedError;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Cannot archive '{src}' to '{dst}': {source}")]
    Archive {
        src: PathBuf,
        dst: String,
        source: std::io::Error,
    },
    #[error(transparent)]
    CommandFailed(#[from] Box<CommandFailedError>),
    #[error("Cannot create git sub-directory at '{0}': {1}")]
    CreateMainDirectory(PathBuf, #[source] std::io::Error),
    #[error("Cannot clean git repo at '{0}' before a re-clone: {1}")]
    CleanSubDirectory(PathBuf, #[source] std::io::Error),
    #[error("Either no base name to url path or it contains non-UTF-8 characters: {0}")]
    NonUtf8BaseName(Url),
}

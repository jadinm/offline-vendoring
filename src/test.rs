//! This module centralizes useful test utils
use flate2::{Compression, write::GzEncoder};
use rstest::fixture;
use tempfile::tempfile;

use crate::ArchiveBuilder;

#[fixture]
pub fn archive() -> ArchiveBuilder {
    let tar_gz = tempfile().unwrap();
    let enc = GzEncoder::new(tar_gz, Compression::default());
    tar::Builder::new(enc)
}

use std::{
    fs::{File, create_dir_all},
    path::{Path, PathBuf},
};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use tar::{Archive, Builder};
use tracing::{debug, info};

use crate::{
    cmd::{CommandRunner, LocalCommandRunner},
    custom::CustomTasks,
    git::GitMirrors,
    python::PythonSettings,
    rust::RustSettings,
};

mod cmd;
mod custom;
mod error;
mod git;
mod python;
mod rust;

pub use error::InstallingError;
pub use error::PackagingError;

const CARGO_TOOLS_PATH: &str = "cargo-tools";
const CARGO_VENDOR_PATH: &str = "cargo-vendor";
const PIP_DOWNLOAD_DIR: &str = "pip";
const MIRRORS_PATH: &str = "mirrors";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default = "default_name")]
    pub name: String,
    pub rust: RustSettings,
    pub python: PythonSettings,
    pub git_mirrors: GitMirrors,
    pub custom: CustomTasks,
}

type ArchiveBuilder = Builder<GzEncoder<File>>;

fn default_name() -> String {
    "offline-vendoring".to_string()
}

/// Download and package external resources listed in the [`Settings`]
///
/// # Errors
///
/// Check [`PackagingError`]
pub fn package(settings: &Settings) -> Result<(), PackagingError> {
    package_inner::<LocalCommandRunner>(settings)
}

fn package_inner<T: CommandRunner>(settings: &Settings) -> Result<(), PackagingError> {
    // Create .tar.gz file
    let tar_gz =
        File::create(format!("{}.tar.gz", settings.name)).map_err(PackagingError::TarStart)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);

    let packaging_directory = PathBuf::from(settings.name.clone());
    if !packaging_directory.exists() {
        create_dir_all(packaging_directory.as_path()).map_err(PackagingError::DirectoryCreation)?;
    }

    settings
        .rust
        .package::<T>(packaging_directory.as_path(), &mut tar)?;
    settings
        .python
        .package::<T>(packaging_directory.as_path(), &mut tar)?;
    settings
        .git_mirrors
        .package::<T>(packaging_directory.as_path(), &mut tar)?;
    settings.custom.package(&mut tar)?;

    // Serialize settings at the root of the archive
    #[expect(clippy::unwrap_used, reason = "should never fail")]
    {
        let temp_dir = tempfile::tempdir().unwrap();
        let settings_file_path = temp_dir.path().join("settings.yaml");
        let settings_file = File::create_new(&settings_file_path).unwrap();
        serde_yaml::to_writer(settings_file, settings).unwrap();
        tar.append_path_with_name(settings_file_path, "settings.yaml")
            .map_err(PackagingError::TarAppend)?;
    }

    tar.finish().map_err(PackagingError::TarFinish)?;

    Ok(())
}

/// Unpack and install the content of the archive at `archive_path` into the current directory
///
/// # Errors
///
/// Check [`InstallingError`]
pub fn install(archive_path: &Path) -> Result<(), InstallingError> {
    install_inner::<LocalCommandRunner>(archive_path)
}

fn install_inner<T: CommandRunner>(archive_path: &Path) -> Result<(), InstallingError> {
    #[expect(clippy::unwrap_used, reason = "CLI checks archive_path points a file")]
    let unpacked_directory = PathBuf::from(archive_path.file_prefix().unwrap());
    if !unpacked_directory.exists() {
        create_dir_all(unpacked_directory.as_path()).map_err(InstallingError::DirectoryCreation)?;
    }

    // Unpack archive .tar.gz
    let tar_gz = File::open(archive_path).map_err(InstallingError::TarUncompress)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive
        .unpack(unpacked_directory.as_path())
        .map_err(InstallingError::TarUncompress)?;
    #[expect(clippy::unwrap_used, reason = "Independent of user input")]
    {
        info!(
            "Archive unpacked to {}",
            unpacked_directory.canonicalize().unwrap().display()
        );
    }

    // Get packaged settings
    let settings_path = unpacked_directory.join("settings");
    let settings = config::Config::builder()
        .add_source(config::File::with_name(&settings_path.display().to_string()).required(false))
        .build()?;
    let settings: Settings = settings
        .try_deserialize()
        .map_err(|e| InstallingError::DeserializeConfig(e.to_string()))?;
    debug!("Got the following settings: {settings:#?}");

    // Install resources
    info!("Installing external resources");
    RustSettings::install(unpacked_directory.as_path())?;
    PythonSettings::install::<T>(unpacked_directory.as_path())?;
    settings
        .git_mirrors
        .install::<T>(unpacked_directory.as_path())?;
    settings.custom.install::<T>(unpacked_directory.as_path())?;

    Ok(())
}

#[cfg(test)]
pub mod test;

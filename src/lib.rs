use std::{
    fs::{File, create_dir_all},
    path::{Path, PathBuf},
};

use clap::ValueEnum;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use tar::{Archive, Builder};
use tracing::{debug, error, info};

use crate::{
    cmd::{CommandRunner, LocalCommandRunner},
    custom::CustomTasks,
    git::GitMirrors,
    python::PythonSettings,
    rust::RustSettings,
};

mod cmd;
mod custom;
mod errors;
mod git;
mod python;
mod rust;

pub use errors::InstallingError;
pub use errors::PackagingError;
pub use python::PythonConfigLevel;

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

#[derive(ValueEnum, Clone, Eq, Hash, PartialEq)]
pub enum DownloadSkip {
    /// Skip rust crate & tool downloading
    Rust,
    /// Skip python package downloading
    Python,
    /// Skip git mirror cloning
    GitClone,
}

#[derive(ValueEnum, Clone, Eq, Hash, PartialEq)]
pub enum InstallSkip {
    /// Skip rust tools install
    RustTools,
    /// Skip rust configuration
    RustConfig,
    /// Skip python configuration
    PythonConfig,
    /// Skip git mirror pushes
    GitPush,
    /// Skip custom tasks
    Custom,
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
pub fn package(settings: &Settings, skip: &[DownloadSkip]) -> Result<(), Box<PackagingError>> {
    package_inner::<LocalCommandRunner>(settings, skip).map_err(Box::new)
}

fn package_inner<T: CommandRunner>(
    settings: &Settings,
    skip: &[DownloadSkip],
) -> Result<(), PackagingError> {
    // Create .tar.gz file
    let tar_gz = File::create(format!("{}.tar.gz", settings.name))
        .map_err(PackagingError::ArchiveCreation)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);

    let packaging_directory = PathBuf::from(settings.name.clone());
    if !packaging_directory.exists() {
        create_dir_all(packaging_directory.as_path())
            .map_err(|e| PackagingError::CreateMainDirectory(packaging_directory.clone(), e))?;
    }
    let packaging_directory = packaging_directory
        .canonicalize()
        .map_err(|e| PackagingError::GetCannonMainDirectory(packaging_directory.clone(), e))?;

    settings.rust.package::<T>(
        packaging_directory.as_path(),
        &mut tar,
        skip.contains(&DownloadSkip::Rust),
    )?;
    settings.python.package::<T>(
        packaging_directory.as_path(),
        &mut tar,
        skip.contains(&DownloadSkip::Python),
    )?;
    settings.git_mirrors.package::<T>(
        packaging_directory.as_path(),
        &mut tar,
        skip.contains(&DownloadSkip::GitClone),
    )?;
    settings.custom.package(&mut tar)?;

    // Serialize settings at the root of the archive
    #[expect(clippy::unwrap_used, reason = "should never fail")]
    {
        let temp_dir = tempfile::tempdir().unwrap();
        let settings_file_path = temp_dir.path().join("settings.yaml");
        let settings_file = File::create_new(&settings_file_path).unwrap();
        serde_yaml::to_writer(settings_file, settings).unwrap();
        tar.append_path_with_name(settings_file_path, "settings.yaml")
            .map_err(PackagingError::ArchiveInsert)?;
    }

    tar.finish().map_err(PackagingError::ArchiveCreation)?;

    Ok(())
}

/// Unpack and install the content of the archive at `archive_path` into the current directory
///
/// # Errors
///
/// Check [`InstallingError`]
pub fn install(
    archive_path: &Path,
    python_config_level: &PythonConfigLevel,
    rust_config_for: Option<&PathBuf>,
    skip: &[InstallSkip],
) -> Result<(), Box<InstallingError>> {
    install_inner::<LocalCommandRunner>(archive_path, python_config_level, rust_config_for, skip)
        .map_err(Box::new)
}

fn install_inner<T: CommandRunner>(
    archive_path: &Path,
    python_config_level: &PythonConfigLevel,
    rust_config_for: Option<&PathBuf>,
    skip: &[InstallSkip],
) -> Result<(), InstallingError> {
    // .file_prefix() isn't available until rust 1.91 and we wish to support rust 1.88 for now
    let archive_base_name = archive_path
        .file_name()
        .map(|name| name.to_string_lossy())
        .ok_or(InstallingError::InvalidArchivePath(
            archive_path.to_path_buf(),
        ))?;
    let archive_prefix =
        archive_base_name
            .split('.')
            .next()
            .ok_or(InstallingError::InvalidArchivePath(
                archive_path.to_path_buf(),
            ))?;

    let unpacked_directory = PathBuf::from(archive_prefix);
    if !unpacked_directory.exists() {
        create_dir_all(unpacked_directory.as_path())
            .map_err(|e| InstallingError::CreateMainDirectory(unpacked_directory.clone(), e))?;
    }
    let unpacked_directory = unpacked_directory
        .canonicalize()
        .map_err(|e| InstallingError::GetCannonMainDirectory(unpacked_directory.clone(), e))?;

    // Unpack archive .tar.gz
    let tar_gz = File::open(archive_path).map_err(InstallingError::ArchiveUncompress)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive
        .unpack(unpacked_directory.as_path())
        .map_err(InstallingError::ArchiveUncompress)?;
    info!("Archive unpacked to {}", unpacked_directory.display());

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
    let mut latest_error: Result<(), _> = Ok(());
    let res_rs = RustSettings::install(unpacked_directory.as_path(), rust_config_for, skip);
    if let Err(ref err) = res_rs {
        error!("Failed to install rust deps: {err}");
        latest_error = res_rs.map_err(InstallingError::Rust);
    }
    if !skip.contains(&InstallSkip::PythonConfig) {
        let res_py =
            PythonSettings::install::<T>(unpacked_directory.as_path(), python_config_level);
        if let Err(ref err) = res_py {
            error!("Failed to install python deps: {err}");
            latest_error = res_py.map_err(InstallingError::Python);
        }
    }
    if !skip.contains(&InstallSkip::GitPush) {
        let res_git = settings
            .git_mirrors
            .install::<T>(unpacked_directory.as_path());
        if let Err(ref err) = res_git {
            error!("Failed to install git deps: {err}");
            latest_error = res_git.map_err(InstallingError::Git);
        }
    }
    if !skip.contains(&InstallSkip::Custom) {
        let res = settings.custom.install::<T>(unpacked_directory.as_path());
        if let Err(ref err) = res {
            error!("Failed to install custom deps: {err}");
            latest_error = res.map_err(InstallingError::Custom);
        }
    }

    latest_error
}

#[cfg(test)]
pub mod test;

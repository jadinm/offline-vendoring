use std::{
    fs::{self, copy},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use toml_edit::{DocumentMut, Item, Table, value};
use tracing::{debug, info};

#[cfg(test)]
mod test;

use crate::{
    ArchiveBuilder, CARGO_TOOLS_PATH, CARGO_VENDOR_PATH, InstallingError, PackagingError,
    cmd::CommandRunner,
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RustSettings {
    manifests: Vec<PathBuf>,
    binaries: Vec<String>,
    /// Some environments might not have access to github.com or other places where rust tools are uploaded
    /// If set to false, a regular "cargo install" is run (much slower than "cargo binstall").
    use_binstall: bool,
}

impl RustSettings {
    fn package_crates<T: CommandRunner>(
        &self,
        out_folder: &Path,
        tar: &mut ArchiveBuilder,
    ) -> Result<(), PackagingError> {
        info!("Packaging rust crates");
        if self.manifests.is_empty() {
            debug!("No crate to package");
            return Ok(());
        }

        let cmd = "cargo";
        let mut args = vec![
            "vendor".to_owned(),
            "--versioned-dirs".to_owned(),
            "--respect-source-config".to_owned(),
        ];
        for manifest in &self.manifests {
            args.push("--sync".to_owned());
            args.push(manifest.display().to_string());
        }
        let out_folder = out_folder.join(CARGO_VENDOR_PATH);
        args.push(out_folder.display().to_string());
        T::run_cmd(cmd, &args, None)?;
        tar.append_dir_all(CARGO_VENDOR_PATH, out_folder)
            .map_err(PackagingError::TarAppend)?;

        Ok(())
    }

    fn package_tools<T: CommandRunner>(
        &self,
        out_folder: &Path,
        tar: &mut ArchiveBuilder,
    ) -> Result<(), PackagingError> {
        info!("Packaging cargo tools");
        if self.binaries.is_empty() {
            debug!("No cargo tool to package");
            return Ok(());
        }

        let out_folder = out_folder.join(CARGO_TOOLS_PATH);
        fs::create_dir_all(&out_folder).map_err(PackagingError::DirectoryCreation)?;

        let cmd = "cargo";
        let mut args = if self.use_binstall {
            vec!["binstall".to_owned(), "--disable-telemetry".to_owned()]
        } else {
            vec!["install".to_owned()]
        };
        args.extend([
            "--root".to_owned(),
            out_folder.display().to_string(),
            "--locked".to_owned(),
        ]);
        for binary in &self.binaries {
            args.push(binary.clone());
        }
        T::run_cmd(cmd, &args, None)?;
        tar.append_dir_all(CARGO_TOOLS_PATH, out_folder)
            .map_err(PackagingError::TarAppend)?;

        Ok(())
    }

    pub(crate) fn package<T: CommandRunner>(
        &self,
        out_folder: &Path,
        tar: &mut ArchiveBuilder,
    ) -> Result<(), PackagingError> {
        self.package_crates::<T>(out_folder, tar)?;
        self.package_tools::<T>(out_folder, tar)
    }

    /// `in_folder` needs to be a canonicalized path
    pub(crate) fn install(in_folder: &Path) -> Result<(), InstallingError> {
        info!("Instructions to configure cargo vendoring");
        let cargo_home = PathBuf::from(
            std::env::var("CARGO_HOME")
                .or(std::env::var("HOME").map(|home| format!("{home}/.cargo")))
                .map_err(InstallingError::NoCargoHome)?,
        );
        info!("Update cargo config to use vendored resources");
        Self::update_cargo_config(
            &cargo_home.join("config.toml"),
            &in_folder.join(CARGO_VENDOR_PATH),
        )?;

        info!("Installing cargo tools");
        let tools_in_folder = in_folder.join(CARGO_TOOLS_PATH);
        if !tools_in_folder.exists() {
            info!("No cargo tool to install");
            return Ok(());
        }

        let tools_paths = fs::read_dir(tools_in_folder).map_err(InstallingError::ReadDirectory)?;
        // Copy rust tools binary
        for src in tools_paths.filter_map(Result::ok) {
            info!("Installing {}", src.path().display());
            let base_name = src.file_name().display().to_string();
            let dst_path = cargo_home.join("bin").join(base_name);
            copy(src.path(), dst_path).map_err(InstallingError::Copy)?;
        }
        Ok(())
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "false positive: toml_edit creates a value if the key doesn't exists"
    )]
    fn update_cargo_config(
        cargo_config: &Path,
        vendored_path: &Path,
    ) -> Result<(), InstallingError> {
        let content = fs::read_to_string(cargo_config).unwrap_or_default();
        let mut doc = content
            .parse::<DocumentMut>()
            .map_err(InstallingError::CargoConfigRead)?;
        let source = doc["source"].or_insert(Item::Table(Table::new()));

        source["crates-io"]["replace-with"] = value("vendored-sources");
        source["vendored-sources"]["directory"] = value(vendored_path.display().to_string());
        debug!(
            "New config for ${{CARGO_HOME}}/config.toml: {}",
            doc.to_string()
        );
        fs::write(cargo_config, doc.to_string()).map_err(InstallingError::CargoConfigWrite)?;
        Ok(())
    }
}

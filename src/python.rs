use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{
    ArchiveBuilder, InstallingError, PIP_DOWNLOAD_DIR, PackagingError, cmd::CommandRunner,
};

#[cfg(test)]
mod test;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PythonSettings {
    requirement_files: Vec<PathBuf>,
}

impl PythonSettings {
    pub(crate) fn package<T: CommandRunner>(
        &self,
        out_folder: &Path,
        tar: &mut ArchiveBuilder,
    ) -> Result<(), PackagingError> {
        info!("Packaging pip wheel packages");
        if self.requirement_files.is_empty() {
            debug!("No python package");
            return Ok(());
        }
        let out_folder = out_folder.join(PIP_DOWNLOAD_DIR);
        fs::create_dir_all(&out_folder).map_err(PackagingError::DirectoryCreation)?;

        for requirement_file in &self.requirement_files {
            let cmd = "pip";
            T::run_cmd(
                cmd,
                &[
                    "download".to_owned(),
                    "-r".to_owned(),
                    requirement_file.display().to_string(),
                    "--dest".to_owned(),
                    out_folder.display().to_string(),
                ],
                None,
            )?;
        }
        tar.append_dir_all(PIP_DOWNLOAD_DIR, out_folder)
            .map_err(PackagingError::TarAppend)?;

        Ok(())
    }

    /// `in_folder` needs to be a canonicalized path
    pub(crate) fn install<T: CommandRunner>(in_folder: &Path) -> Result<(), InstallingError> {
        info!(
            "Configuring pip (user-level) to look at those wheel packages and never at the index"
        );
        let in_folder = in_folder.join(PIP_DOWNLOAD_DIR);
        // Tell all pip run with that user to use the extracted folder
        T::run_cmd(
            "pip",
            &[
                "config".to_owned(),
                "--user".to_owned(),
                "set".to_owned(),
                "global.find-links".to_owned(),
                in_folder.display().to_string(),
            ],
            None,
        )?;
        // Faster if we disable any request to pypi website
        T::run_cmd(
            "pip",
            &[
                "config".to_owned(),
                "--user".to_owned(),
                "set".to_owned(),
                "global.no-index".to_owned(),
                "true".to_owned(),
            ],
            None,
        )?;
        Ok(())
    }
}

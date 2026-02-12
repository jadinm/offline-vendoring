use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use url::Url;

use crate::{ArchiveBuilder, InstallingError, MIRRORS_PATH, PackagingError, cmd::CommandRunner};

#[cfg(test)]
mod test;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GitMirrors {
    mirrors: Vec<GitMirror>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitMirror {
    src: Url,
    dst: Url,
}

impl GitMirrors {
    pub(crate) fn package<T: CommandRunner>(
        &self,
        out_folder: &Path,
        tar: &mut ArchiveBuilder,
    ) -> Result<(), PackagingError> {
        info!("Packaging git mirrors");
        if self.mirrors.is_empty() {
            debug!("No mirror to clone");
            return Ok(());
        }
        let out_folder = out_folder.join(MIRRORS_PATH);
        fs::create_dir_all(&out_folder).map_err(PackagingError::DirectoryCreation)?;

        for mirror in &self.mirrors {
            let mirror_basename = PathBuf::from(mirror.src.path())
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .ok_or(PackagingError::InvalidCharacter(mirror.src.clone()))?
                .to_owned();

            T::run_cmd(
                "git",
                &[
                    "clone".to_owned(),
                    "--mirror".to_owned(),
                    mirror.src.to_string(),
                    out_folder.join(mirror_basename).display().to_string(),
                ],
                None,
            )?;
        }
        tar.append_dir_all(MIRRORS_PATH, out_folder)
            .map_err(PackagingError::TarAppend)?;
        Ok(())
    }

    /// `in_folder` needs to be a canonicalized path
    pub(crate) fn install<T: CommandRunner>(
        &self,
        in_folder: &Path,
    ) -> Result<(), InstallingError> {
        info!("Synching git mirrors");
        let in_folder = in_folder.join(MIRRORS_PATH);
        for mirror in &self.mirrors {
            let mirror_basename = PathBuf::from(mirror.src.path())
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .ok_or(InstallingError::InvalidCharacter(mirror.src.clone()))?
                .to_owned();
            let in_folder = in_folder.join(mirror_basename);

            T::run_cmd(
                "git",
                &[
                    "push".to_owned(),
                    "--mirror".to_owned(),
                    mirror.dst.to_string(),
                ],
                Some(in_folder),
            )?;
        }
        Ok(())
    }
}

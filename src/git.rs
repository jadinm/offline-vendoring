use std::{
    fs::{self, remove_dir_all},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use url::Url;

use crate::{ArchiveBuilder, MIRRORS_PATH, cmd::CommandRunner, git::errors::GitError};

pub mod errors;
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
        skip_download: bool,
    ) -> Result<(), GitError> {
        info!("Packaging git mirrors");
        if self.mirrors.is_empty() {
            debug!("No mirror to clone");
            return Ok(());
        }
        let out_folder = out_folder.join(MIRRORS_PATH);
        fs::create_dir_all(&out_folder)
            .map_err(|e| GitError::CreateMainDirectory(out_folder.clone(), e))?;

        if !skip_download {
            for mirror in &self.mirrors {
                let mirror_basename = PathBuf::from(mirror.src.path())
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                    .ok_or(GitError::NonUtf8BaseName(mirror.src.clone()))?
                    .to_owned();

                let mirror_clone_path = out_folder.join(&mirror_basename);
                if mirror_clone_path.exists() && mirror_clone_path.is_dir() {
                    remove_dir_all(&mirror_clone_path)
                        .map_err(|e| GitError::CleanSubDirectory(out_folder.clone(), e))?;
                }

                T::run_cmd(
                    "git",
                    &[
                        "clone".to_owned(),
                        "--mirror".to_owned(),
                        mirror.src.to_string(),
                        mirror_basename,
                    ],
                    Some(out_folder.clone()),
                )?;
            }
        }
        tar.append_dir_all(MIRRORS_PATH, &out_folder)
            .map_err(|e| GitError::Archive {
                src: out_folder,
                dst: MIRRORS_PATH.to_owned(),
                source: e,
            })?;
        Ok(())
    }

    /// `in_folder` needs to be a canonicalized path
    pub(crate) fn install<T: CommandRunner>(&self, in_folder: &Path) -> Result<(), GitError> {
        info!("Synching git mirrors");
        let in_folder = in_folder.join(MIRRORS_PATH);
        for mirror in &self.mirrors {
            let mirror_basename = PathBuf::from(mirror.src.path())
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .ok_or(GitError::NonUtf8BaseName(mirror.src.clone()))?
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

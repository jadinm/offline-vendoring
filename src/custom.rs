use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::{ArchiveBuilder, InstallingError, PackagingError, cmd::CommandRunner};

#[cfg(test)]
mod test;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CustomTasks {
    tasks: Vec<CustomTask>,
}

#[derive(Debug, Deserialize, Serialize)]
#[expect(clippy::doc_markdown, reason = "false positives")]
pub struct CustomTask {
    /// An hashmap mapping local paths (to files or directories) to their relative path within the archive
    /// e.g., {"/path/to/vscode/code-spell-check.vsix": "vscode/code-spell-check.vsix", "/path/to/debian_packages": "debian_packages"}
    paths_to_package: HashMap<PathBuf, PathBuf>,
    /// An optional command to install the files on the offline machine.
    /// e.g., "code --install-extension" (with `install_counts` set to "EachFile")
    /// The working directory of the command will be the root of the extracted archive.
    install_command: Option<String>,
    /// How many times to run the install command (see [`CustomInstallInstallCount`])
    install_counts: CustomInstallInstallCount,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum CustomInstallInstallCount {
    /// The custom command must be run once for each path, the last argument will be the path
    EachPath,
    /// The custom command must be run once for each file (directly listed or within a listed directory),
    /// the last argument will be the path to the file
    EachFile,
    /// The custom command must be run only once and no extra argument will be added
    #[default]
    Once,
}

impl CustomTasks {
    pub(crate) fn package(&self, tar: &mut ArchiveBuilder) -> Result<(), PackagingError> {
        info!("Packaging custom tasks");
        for task in &self.tasks {
            debug!("Processing {:#?}", task);
            for (local_path, package_path) in &task.paths_to_package {
                debug!("Processing {}", local_path.display());
                if local_path.is_dir() {
                    tar.append_dir_all(package_path, local_path)
                        .map_err(PackagingError::TarAppend)?;
                } else {
                    tar.append_path_with_name(local_path, package_path)
                        .map_err(PackagingError::TarAppend)?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn install<T: CommandRunner>(
        &self,
        in_folder: &Path,
    ) -> Result<(), InstallingError> {
        info!("Installing through custom tasks");
        for task in &self.tasks {
            if task.install_command.is_none() {
                continue;
            }
            #[expect(clippy::unwrap_used, reason = "already checked")]
            let install_command = task.install_command.as_ref().unwrap();
            let install_command = shlex::split(install_command).ok_or(
                InstallingError::CustomInstallCommand(install_command.clone()),
            )?;
            if install_command.is_empty() {
                warn!("Empty string command in task '{task:#?}'");
                continue;
            }

            match task.install_counts {
                CustomInstallInstallCount::EachPath => {
                    for package_path in task.paths_to_package.values() {
                        let package_path = in_folder.join(package_path);
                        Self::run_command_with_path::<T>(
                            &install_command,
                            &package_path,
                            in_folder.to_path_buf(),
                        )?;
                    }
                }
                CustomInstallInstallCount::EachFile => {
                    for package_path in task.paths_to_package.values() {
                        let package_path = in_folder.join(package_path);

                        // Recursively look over every file in listed folders
                        if package_path.is_dir() {
                            for entry in WalkDir::new(&package_path) {
                                let entry = entry.map_err(InstallingError::WalkDirectory)?;
                                if entry.path().is_file() {
                                    Self::run_command_with_path::<T>(
                                        &install_command,
                                        entry.path(),
                                        in_folder.to_path_buf(),
                                    )?;
                                }
                            }
                        } else {
                            Self::run_command_with_path::<T>(
                                &install_command,
                                &package_path,
                                in_folder.to_path_buf(),
                            )?;
                        }
                    }
                }
                CustomInstallInstallCount::Once => {
                    #[expect(clippy::indexing_slicing, reason = "checked after shlex.split()")]
                    T::run_cmd(
                        &install_command[0],
                        &install_command[1..],
                        Some(in_folder.to_path_buf()),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn run_command_with_path<T: CommandRunner>(
        install_command: &[String],
        extra_arg_path: &Path,
        cwd: PathBuf,
    ) -> Result<(), InstallingError> {
        let mut args_with_path = install_command.to_vec();
        args_with_path.push(extra_arg_path.display().to_string());
        #[expect(clippy::indexing_slicing, reason = "checked after shlex.split()")]
        T::run_cmd(&install_command[0], &args_with_path[1..], Some(cwd))?;

        Ok(())
    }
}

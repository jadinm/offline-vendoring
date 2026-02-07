use std::{path::PathBuf, process::Command};

use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum CommandFailedError {
    #[error("Failed to start command: {0}")]
    CommandStart(#[from] std::io::Error),
    #[error("Command exited with error code {0}")]
    CommandFailed(std::process::ExitStatus),
}

/// This trait's purpose is mainly to be able to capture its implementation in mockall
#[cfg_attr(test, mockall::automock)]
pub(crate) trait CommandRunner {
    fn run_cmd(cmd: &str, args: &[String], cwd: Option<PathBuf>) -> Result<(), CommandFailedError> {
        info!("Running '{cmd} {}'", args.join(" "));
        let mut cmd = Command::new(cmd);
        cmd.args(args);
        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }
        let status = cmd.status()?;
        if !status.success() {
            return Err(CommandFailedError::CommandFailed(status));
        }
        Ok(())
    }
}

/// A structure running requested command on the local machine
pub(crate) struct LocalCommandRunner;

impl CommandRunner for LocalCommandRunner {
    fn run_cmd(cmd: &str, args: &[String], cwd: Option<PathBuf>) -> Result<(), CommandFailedError> {
        info!("Running '{cmd} {}'", args.join(" "));
        let mut cmd = Command::new(cmd);
        cmd.args(args);
        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }
        let status = cmd.status()?;
        if !status.success() {
            return Err(CommandFailedError::CommandFailed(status));
        }
        Ok(())
    }
}

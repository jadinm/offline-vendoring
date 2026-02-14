use std::{path::PathBuf, process::Command};

use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum CommandFailedError {
    #[error("Failed to start command '{0:#?}': {1}")]
    CommandStart(Command, #[source] std::io::Error),
    #[error("Command '{0:#?}' exited with error code {1}")]
    CommandFailed(Command, std::process::ExitStatus),
}

/// This trait's purpose is mainly to be able to capture its implementation in mockall
#[cfg_attr(test, mockall::automock)]
pub(crate) trait CommandRunner {
    fn run_cmd(
        cmd: &str,
        args: &[String],
        cwd: Option<PathBuf>,
    ) -> Result<(), Box<CommandFailedError>>;
}

/// A structure running requested command on the local machine
pub(crate) struct LocalCommandRunner;

impl CommandRunner for LocalCommandRunner {
    fn run_cmd(
        cmd: &str,
        args: &[String],
        cwd: Option<PathBuf>,
    ) -> Result<(), Box<CommandFailedError>> {
        info!("Running '{cmd} {}'", args.join(" "));
        let mut cmd = Command::new(cmd);
        cmd.args(args);
        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }
        let status = match cmd.status() {
            Ok(status) => status,
            Err(e) => return Err(Box::new(CommandFailedError::CommandStart(cmd, e))),
        };
        if !status.success() {
            return Err(Box::new(CommandFailedError::CommandFailed(cmd, status)));
        }
        Ok(())
    }
}

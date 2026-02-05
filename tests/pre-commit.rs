use std::{
    io::{self, Write as _},
    process::Command,
};

use tracing::info;

#[test_log::test]
/// Validate in automated tests that pre-commit scripts were run locally
fn check_pre_commit_scripts() {
    let output = Command::new("pre-commit")
        .args(["run", "-a"])
        .output()
        .expect("failed to execute pre-commit");

    info!("pre-commit returned: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());
}

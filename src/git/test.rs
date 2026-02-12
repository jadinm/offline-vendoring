use std::sync::Mutex;

use mockall::predicate::{eq, function};
use rstest::rstest;
use tempfile::tempdir;

use crate::MIRRORS_PATH;
use crate::cmd::LocalCommandRunner;
use crate::git::GitMirrors;
use crate::test::archive;
use crate::{ArchiveBuilder, cmd::MockCommandRunner};

/// Required to lock this mutex in every test
/// because of <https://docs.rs/mockall/latest/mockall/#static-methods>
///
/// The mutex might be poisoned if a test fails. But we don't
/// care, because it doesn't hold any data. Whether it's poisoned or
/// not, we'll still hold the `MutexGuard`.
static MTX: Mutex<()> = Mutex::new(());

#[rstest]
#[test_log::test]
fn package_empty(mut archive: ArchiveBuilder) {
    let git: GitMirrors = serde_yaml::from_str(
        "
mirrors: []
",
    )
    .unwrap();

    let out_folder = tempdir().unwrap();
    git.package::<LocalCommandRunner>(out_folder.path(), &mut archive, false)
        .expect("Shouldn't fail to package crates");
    archive.finish().expect("Shouldn't fail to build archive");
}

#[rstest]
#[test_log::test]
fn package(mut archive: ArchiveBuilder) {
    let git: GitMirrors = serde_yaml::from_str(
        "
mirrors:
    - src: https://github.com/doublify/pre-commit-rust
      dst: https://private.domain/global/pre-commit-rust
    - src: https://github.com/rustsec/advisory-db
      dst: https://private.domain/global/advisory-db
",
    )
    .unwrap();

    let out_folder = tempdir().unwrap();
    git.package::<LocalCommandRunner>(out_folder.path(), &mut archive, false)
        .expect("Shouldn't fail to install mirrors");
    archive.finish().expect("Shouldn't fail to build archive");

    // Check the clones
    let mirror_path = out_folder.path().join(MIRRORS_PATH);
    assert!(mirror_path.join("pre-commit-rust").exists());
    assert!(mirror_path.join("advisory-db").exists());
}

#[test_log::test]
fn install() {
    let _m = MTX.lock();

    let git: GitMirrors = serde_yaml::from_str(
        "
mirrors:
    - src: https://github.com/doublify/pre-commit-rust
      dst: https://private.domain/global/pre-commit-rust
    - src: https://github.com/rustsec/advisory-db
      dst: https://private.domain/global/advisory-db
",
    )
    .unwrap();

    let ctx = MockCommandRunner::run_cmd_context();
    let in_folder = tempdir().unwrap();
    let mirrors_folder = in_folder.path().join(MIRRORS_PATH);

    let pre_commit_rust = mirrors_folder.join("pre-commit-rust");
    ctx.expect()
        .with(
            eq("git"),
            function(move |args: &[String]| {
                args == [
                    "push".to_owned(),
                    "--mirror".to_owned(),
                    "https://private.domain/global/pre-commit-rust".to_owned(),
                ]
            }),
            eq(Some(pre_commit_rust)),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));
    let advisory_db = mirrors_folder.join("advisory-db");
    ctx.expect()
        .with(
            eq("git"),
            function(move |args: &[String]| {
                args == [
                    "push".to_owned(),
                    "--mirror".to_owned(),
                    "https://private.domain/global/advisory-db".to_owned(),
                ]
            }),
            eq(Some(advisory_db)),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));

    git.install::<MockCommandRunner>(in_folder.path())
        .expect("Shouldn't fail to install mirrors");
}

use std::fs::{self, File, create_dir_all};
use std::io::Write;
use std::sync::Mutex;

use assertables::assert_fs_read_to_string_eq_x;
use mockall::predicate::{eq, function};
use rstest::rstest;
use tempfile::tempdir;
use tracing::info;

use crate::test::archive;
use crate::{ArchiveBuilder, cmd::MockCommandRunner, rust::RustSettings};
use crate::{CARGO_TOOLS_PATH, CARGO_VENDOR_PATH};

/// Required to lock this mutex in every test
/// because of <https://docs.rs/mockall/latest/mockall/#static-methods>
///
/// The mutex might be poisoned if a test fails. But we don't
/// care, because it doesn't hold any data. Whether it's poisoned or
/// not, we'll still hold the `MutexGuard`.
static MTX: Mutex<()> = Mutex::new(());

#[rstest]
#[test_log::test]
fn package_crates(mut archive: ArchiveBuilder) {
    let _m = MTX.lock();

    let ctx = MockCommandRunner::run_cmd_context();
    let out_folder = tempdir().unwrap();
    let crate_folder = out_folder.path().join(CARGO_VENDOR_PATH);
    let crate_folder_clone = crate_folder.clone();
    ctx.expect()
        .with(
            eq("cargo"),
            function(move |args: &[String]| {
                args.ends_with(&[
                    "--sync".to_owned(),
                    "./Cargo.toml".to_owned(),
                    crate_folder.display().to_string(),
                ])
            }),
            eq(None),
        )
        .times(1)
        .returning(move |_, _, _| {
            // cargo vendor creates the folder and it's expected by our tested code
            fs::create_dir_all(&crate_folder_clone).unwrap();
            Ok(())
        });

    let rust: RustSettings = serde_yaml::from_str(
        "
manifests:
    - ./Cargo.toml
binaries: []
use_binstall: true
",
    )
    .unwrap();

    rust.package_crates::<MockCommandRunner>(out_folder.path(), &mut archive, false)
        .expect("Shouldn't fail to package crates");
    archive.finish().expect("Shouldn't fail to build archive");
}

#[rstest]
#[test_log::test]
fn package_tools(mut archive: ArchiveBuilder) {
    let _m = MTX.lock();

    let ctx = MockCommandRunner::run_cmd_context();

    let out_folder = tempdir().unwrap();
    let binary_folder = out_folder.path().join(CARGO_TOOLS_PATH);
    ctx.expect()
        .with(
            eq("cargo"),
            function(move |args: &[String]| {
                args.ends_with(&[
                    "--root".to_owned(),
                    binary_folder.display().to_string(),
                    "--locked".to_owned(),
                    "cargo-audit".to_owned(),
                    "cargo-deny".to_owned(),
                ])
            }),
            eq(None),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));

    let rust: RustSettings = serde_yaml::from_str(
        "
manifests: []
binaries:
    - cargo-audit
    - cargo-deny
use_binstall: true
",
    )
    .unwrap();

    rust.package_tools::<MockCommandRunner>(out_folder.path(), &mut archive, false)
        .expect("Shouldn't fail to package tools");
    archive.finish().expect("Shouldn't fail to build archive");
}

#[rstest]
#[test_log::test]
fn package_empty_lists(mut archive: ArchiveBuilder) {
    let _m = MTX.lock();

    let ctx = MockCommandRunner::run_cmd_context();
    ctx.expect().never();

    let rust: RustSettings = serde_yaml::from_str(
        "
manifests: []
binaries: []
use_binstall: true
",
    )
    .unwrap();

    let out_folder = tempdir().unwrap();
    rust.package::<MockCommandRunner>(out_folder.path(), &mut archive, false)
        .expect("Shouldn't fail because there is no listed resources");
    archive.finish().expect("Shouldn't fail to build archive");
}

#[rstest]
#[case::void_input(
    None,
    r#"[source]
crates-io = { replace-with = "vendored-sources" }
vendored-sources = { directory = _DIRECTORY_ }
"#,
    vec![]
)]
#[case::empty_input(
    Some(""),
    r#"[source]
crates-io = { replace-with = "vendored-sources" }
vendored-sources = { directory = _DIRECTORY_ }
"#,
    vec![]
)]
#[case::non_overlapping_content_plus_one_tool(
    Some(r#"[global]
credential-provider = "cargo:token"
"#),
    r#"[global]
credential-provider = "cargo:token"

[source]
crates-io = { replace-with = "vendored-sources" }
vendored-sources = { directory = _DIRECTORY_ }
"#,
    vec!["cargo-audit"]
)]
#[case::overlapping_content_plus_two_tools(
    Some(r#"[global]
credential-provider = "cargo:token"

[source]
crates-io = { replace-with = "remote-sources" }
vendored-sources = { directory = 'lol' }
"#),
    r#"[global]
credential-provider = "cargo:token"

[source]
crates-io = { replace-with = "vendored-sources" }
vendored-sources = { directory = _DIRECTORY_ }
"#,
    vec!["cargo-audit", "cargo-deny"]
)]
#[case::overlapping_content_alternate_notation_plus_two_tools(
    Some(r#"[global]
credential-provider = "cargo:token"

[source.crates-io]
replace-with = "remote-sources"

[source.vendored-sources]
directory = 'lol'
"#),
    r#"[global]
credential-provider = "cargo:token"

[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = _DIRECTORY_
"#,
    vec!["cargo-audit", "cargo-deny"]
)]
#[test_log::test]
fn install(
    #[case] initial_config_toml: Option<&str>,
    #[case] expected_config_toml: &str,
    #[case] tools: Vec<&str>,
) {
    // set_var is only safe to call in single-threaded environment, so we need a lock
    let _m = MTX.lock();
    let out_folder = tempdir().unwrap();
    temp_env::with_var(
        "CARGO_HOME",
        Some(out_folder.path().display().to_string()),
        || {
            // Create binary folder within $CARGO_HOME, expected to be there in any sane rust setup
            create_dir_all(out_folder.path().join("bin")).unwrap();

            // Set original content if any
            let config_toml_path = out_folder.path().join("config.toml");
            if let Some(initial_config_toml) = initial_config_toml {
                let mut config = File::create(&config_toml_path).unwrap();
                write!(config, "{initial_config_toml}").unwrap();
            }

            // Create dummy tools
            let in_folder = tempdir().unwrap();
            if !tools.is_empty() {
                create_dir_all(in_folder.path().join(CARGO_TOOLS_PATH).join("bin")).unwrap();
            }
            for tool in &tools {
                let tool_path = in_folder
                    .path()
                    .join(CARGO_TOOLS_PATH)
                    .join("bin")
                    .join(tool);
                info!("Creating file {}", tool_path.display());
                File::create(tool_path).unwrap();
            }

            // Actual tested operation
            RustSettings::install(in_folder.path(), None, &[]).expect("Installation failed");

            // Both OS have different way of quoting paths
            #[cfg(target_os = "linux")]
            let quote = "\"";
            #[cfg(target_os = "windows")]
            let quote = "'";
            // Check for successful cargo configuration
            let expected_config_toml = expected_config_toml.replace(
                "_DIRECTORY_",
                &format!(
                    "{quote}{}{quote}",
                    &in_folder
                        .path()
                        .join(CARGO_VENDOR_PATH)
                        .display()
                        .to_string()
                ),
            );
            assert_fs_read_to_string_eq_x!(
                config_toml_path,
                expected_config_toml,
                "Config.toml should have expected content"
            );

            // Check for successful cargo tool install
            for tool in &tools {
                assert_fs_read_to_string_eq_x!(
                    out_folder.path().join("bin").join(tool),
                    "", // Empty files
                    "The file should exist with the same content, that is, an empty string"
                );
            }
        },
    );
}

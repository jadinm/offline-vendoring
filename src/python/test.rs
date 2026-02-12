use std::fs::{self, File};
use std::io::Write;

use rstest::rstest;
use tempfile::tempdir;

use crate::{
    ArchiveBuilder, PIP_DOWNLOAD_DIR, cmd::LocalCommandRunner, python::PythonSettings,
    test::archive,
};

#[rstest]
#[test_log::test]
fn package_empty_lists(mut archive: ArchiveBuilder) {
    let python: PythonSettings = serde_yaml::from_str(
        "
requirement_files: []
",
    )
    .unwrap();

    let out_folder = tempdir().unwrap();
    python
        .package::<LocalCommandRunner>(out_folder.path(), &mut archive, false)
        .expect("Shouldn't fail because there is no listed resources");
    archive.finish().expect("Shouldn't fail to build archive");
}

#[rstest]
#[test_log::test]
fn package_regular(mut archive: ArchiveBuilder) {
    // Create requirement file
    let in_folder = tempdir().unwrap();
    let requirements_path = in_folder.path().join("requirements.txt");
    let mut requirements = File::create_new(&requirements_path).unwrap();
    write!(requirements, "pre-commit").unwrap();
    let python: PythonSettings = serde_json::from_value(
        serde_json::json!({"requirement_files": [requirements_path.display().to_string()]}),
    )
    .unwrap();

    let out_folder = tempdir().unwrap();
    python
        .package::<LocalCommandRunner>(out_folder.path(), &mut archive, false)
        .expect("Shouldn't fail to package python resources");
    archive.finish().expect("Shouldn't fail to build archive");

    let download_wheels = fs::read_dir(out_folder.path().join(PIP_DOWNLOAD_DIR))
        .expect("No pip download folder created");
    for wheel_path in download_wheels.filter_map(Result::ok) {
        if wheel_path
            .file_name()
            .display()
            .to_string()
            .contains("pre_commit")
        {
            return;
        }
    }
    panic!("pre-commit wasn't found in the pip download folder");
}

// No tests on install process because impossible to change user-level config location in windows

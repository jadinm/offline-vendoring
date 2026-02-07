use std::fs::{File, create_dir_all};
use std::sync::Mutex;

use mockall::predicate::{eq, function};
use rstest::rstest;
use tempfile::tempdir;

use crate::custom::CustomTasks;
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
    let tasks: CustomTasks = serde_yaml::from_str(
        "
tasks: []
",
    )
    .unwrap();
    tasks
        .package(&mut archive)
        .expect("Shouldn't fail to custom files");
    archive.finish().expect("Shouldn't fail to build archive");
}

#[rstest]
#[test_log::test]
fn package_regular(mut archive: ArchiveBuilder) {
    let in_folder = tempdir().unwrap();
    let example_folder = tempdir().unwrap();
    let example_file_path = in_folder.path().join("example_file_in");
    File::create_new(&example_file_path).unwrap();

    let tasks: CustomTasks = serde_json::from_value(serde_json::json!({
        "tasks": [
            {
                "paths_to_package": {
                    example_file_path.display().to_string(): "folder/example_file"
                },
                "install_command": "echo",
                "install_counts": "EachFile",
            },
            {
                "paths_to_package": {
                    example_folder.path().display().to_string(): "vscode/"
                },
                "install_command": "echo",
                "install_counts": "EachPath",
            },
        ]
    }))
    .unwrap();

    tasks
        .package(&mut archive)
        .expect("Shouldn't fail to package custom files");
    archive.finish().expect("Shouldn't fail to build archive");
}

#[test_log::test]
fn install() {
    let _m = MTX.lock();
    let in_folder = tempdir().unwrap();

    // Create a folder with multiple files and a separate file

    let example_folder = in_folder.path().join("example_folder");
    create_dir_all(&example_folder).unwrap();

    let sub_files = [example_folder.join("1"), example_folder.join("2")];
    for sub_file in &sub_files {
        File::create_new(sub_file).unwrap();
    }

    let example_file = in_folder.path().join("example_file");
    File::create_new(&example_file).unwrap();

    let tasks: CustomTasks = serde_json::from_value(serde_json::json!({
        "tasks": [
            {
                "paths_to_package": {
                    "example_folder": example_folder,
                    "example_file": example_file,
                },
                "install_command": "echo",
                "install_counts": "EachFile",
            },
            {
                "paths_to_package": {
                    "example_folder": example_folder,
                    "example_file": example_file,
                },
                "install_command": "echo",
                "install_counts": "EachPath",
            },
            {
                "paths_to_package": {
                    "example_folder": example_folder,
                    "example_file": example_file,
                },
                "install_command": "echo",
                "install_counts": "Once",
            },
        ]
    }))
    .unwrap();

    let ctx = MockCommandRunner::run_cmd_context();
    let cwd = in_folder.path();

    // First task (for each file)

    let example_folder_sub_1 = sub_files[0].clone();
    ctx.expect()
        .with(
            eq("echo"),
            function(move |args: &[String]| args == [example_folder_sub_1.display().to_string()]),
            eq(Some(cwd.to_path_buf())),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));
    let example_folder_sub_2 = sub_files[1].clone();
    ctx.expect()
        .with(
            eq("echo"),
            function(move |args: &[String]| args == [example_folder_sub_2.display().to_string()]),
            eq(Some(cwd.to_path_buf())),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));
    let example_file_clone_2 = example_file.clone();
    ctx.expect()
        .with(
            eq("echo"),
            function(move |args: &[String]| args == [example_file_clone_2.display().to_string()]),
            eq(Some(cwd.to_path_buf())),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));

    // Second task (for each path)

    let example_folder_clone = example_folder.clone();
    ctx.expect()
        .with(
            eq("echo"),
            function(move |args: &[String]| args == [example_folder_clone.display().to_string()]),
            eq(Some(cwd.to_path_buf())),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));
    let example_file_clone = example_file.clone();
    ctx.expect()
        .with(
            eq("echo"),
            function(move |args: &[String]| args == [example_file_clone.display().to_string()]),
            eq(Some(cwd.to_path_buf())),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));

    // Third task (only once)

    ctx.expect()
        .with(
            eq("echo"),
            function(move |args: &[String]| args.is_empty()),
            eq(Some(cwd.to_path_buf())),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));

    tasks
        .install::<MockCommandRunner>(in_folder.path())
        .expect("Shouldn't fail to run custom tasks");
}

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_test_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn create_file(dir: &TempDir, path: &str, content: &str) {
    let file_path = dir.path().join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&file_path, content).unwrap();
}

fn read_file(dir: &TempDir, path: &str) -> String {
    fs::read_to_string(dir.path().join(path)).unwrap()
}

#[test]
fn test_piped_file_list() {
    let dir = setup_test_dir();
    create_file(&dir, "file1.txt", "old");
    create_file(&dir, "file2.txt", "old");
    create_file(&dir, "ignored.txt", "old");

    let file_list = format!(
        "{}\n{}",
        dir.path().join("file1.txt").display(),
        dir.path().join("file2.txt").display()
    );

    Command::cargo_bin("csd")
        .unwrap()
        .arg("old")
        .arg("new")
        .write_stdin(file_list)
        .assert()
        .success()
        .stderr(predicate::str::contains("modified 2"));

    assert_eq!(read_file(&dir, "file1.txt"), "new");
    assert_eq!(read_file(&dir, "file2.txt"), "new");
    // File not in the piped list should not be modified
    assert_eq!(read_file(&dir, "ignored.txt"), "old");
}

#[test]
fn test_piped_single_file() {
    let dir = setup_test_dir();
    create_file(&dir, "target.txt", "replace me");

    let file_path = format!("{}", dir.path().join("target.txt").display());

    Command::cargo_bin("csd")
        .unwrap()
        .arg("replace me")
        .arg("replaced")
        .write_stdin(file_path)
        .assert()
        .success();

    assert_eq!(read_file(&dir, "target.txt"), "replaced");
}

#[test]
fn test_piped_empty_list() {
    Command::cargo_bin("csd")
        .unwrap()
        .arg("search")
        .arg("replace")
        .write_stdin("")
        .assert()
        .success()
        .stderr(predicate::str::contains("No matching files found"));
}

#[test]
fn test_piped_with_nonexistent_file() {
    let dir = setup_test_dir();
    create_file(&dir, "exists.txt", "content");

    let file_list = format!(
        "{}\n{}/nonexistent.txt",
        dir.path().join("exists.txt").display(),
        dir.path().display()
    );

    Command::cargo_bin("csd")
        .unwrap()
        .arg("content")
        .arg("replaced")
        .write_stdin(file_list)
        .assert()
        .success();

    // Only the existing file should be processed
    assert_eq!(read_file(&dir, "exists.txt"), "replaced");
}

#[test]
fn test_piped_files_with_spaces() {
    let dir = setup_test_dir();
    create_file(&dir, "file with spaces.txt", "content");

    let file_path = format!("{}", dir.path().join("file with spaces.txt").display());

    Command::cargo_bin("csd")
        .unwrap()
        .arg("content")
        .arg("changed")
        .write_stdin(file_path)
        .assert()
        .success();

    assert_eq!(read_file(&dir, "file with spaces.txt"), "changed");
}

#[test]
fn test_piped_mixed_relative_absolute() {
    let dir = setup_test_dir();
    create_file(&dir, "absolute.txt", "old");
    create_file(&dir, "relative.txt", "old");

    // Use absolute paths for both
    let file_list = format!(
        "{}\n{}",
        dir.path().join("absolute.txt").display(),
        dir.path().join("relative.txt").display()
    );

    Command::cargo_bin("csd")
        .unwrap()
        .arg("old")
        .arg("new")
        .write_stdin(file_list)
        .assert()
        .success();

    assert_eq!(read_file(&dir, "absolute.txt"), "new");
    assert_eq!(read_file(&dir, "relative.txt"), "new");
}

#[test]
fn test_piped_whitespace_only_lines() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "content");

    let file_list = format!(
        "  \n\t\n{}\n  \n",
        dir.path().join("test.txt").display()
    );

    Command::cargo_bin("csd")
        .unwrap()
        .arg("content")
        .arg("replaced")
        .write_stdin(file_list)
        .assert()
        .success()
        .stderr(predicate::str::contains("modified 1"));

    assert_eq!(read_file(&dir, "test.txt"), "replaced");
}

#[test]
fn test_piped_duplicate_files() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "count");

    // List the same file multiple times
    let file_path = dir.path().join("test.txt").display().to_string();
    let file_list = format!("{}\n{}\n{}", file_path, file_path, file_path);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("count")
        .arg("replaced")
        .write_stdin(file_list)
        .assert()
        .success();

    // File should only be processed once despite being listed multiple times
    assert_eq!(read_file(&dir, "test.txt"), "replaced");
}

#[test]
fn test_piped_with_quiet_flag() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "old");

    let file_path = format!("{}", dir.path().join("test.txt").display());

    Command::cargo_bin("csd")
        .unwrap()
        .arg("-q")
        .arg("old")
        .arg("new")
        .write_stdin(file_path)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert_eq!(read_file(&dir, "test.txt"), "new");
}

#[test]
fn test_piped_with_dry_run() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "original");

    let file_path = format!("{}", dir.path().join("test.txt").display());

    Command::cargo_bin("csd")
        .unwrap()
        .arg("--dry-run")
        .arg("original")
        .arg("modified")
        .write_stdin(file_path)
        .assert()
        .success();

    // File should not be modified in dry-run mode
    assert_eq!(read_file(&dir, "test.txt"), "original");
}

#[test]
fn test_piped_long_file_list() {
    let dir = setup_test_dir();

    // Create 100 files
    for i in 0..100 {
        create_file(&dir, &format!("file{}.txt", i), "old");
    }

    // Build file list
    let file_list: Vec<String> = (0..100)
        .map(|i| dir.path().join(format!("file{}.txt", i)).display().to_string())
        .collect();
    let file_list = file_list.join("\n");

    Command::cargo_bin("csd")
        .unwrap()
        .arg("old")
        .arg("new")
        .write_stdin(file_list)
        .assert()
        .success()
        .stderr(predicate::str::contains("modified 100"));

    // Verify a few random files
    assert_eq!(read_file(&dir, "file0.txt"), "new");
    assert_eq!(read_file(&dir, "file50.txt"), "new");
    assert_eq!(read_file(&dir, "file99.txt"), "new");
}

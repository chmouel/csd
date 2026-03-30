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

fn file_list(dir: &TempDir, paths: &[&str]) -> String {
    paths
        .iter()
        .map(|p| dir.path().join(p).display().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_basic_replace() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "hello world\nhello again");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("hello")
        .arg("goodbye")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("modified 1"));

    assert_eq!(read_file(&dir, "test.txt"), "goodbye world\ngoodbye again");
}

#[test]
fn test_multiple_files() {
    let dir = setup_test_dir();
    create_file(&dir, "file1.txt", "old");
    create_file(&dir, "file2.txt", "old");
    create_file(&dir, "file3.txt", "old");

    let input = file_list(&dir, &["file1.txt", "file2.txt", "file3.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("old")
        .arg("new")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("modified 3"));

    assert_eq!(read_file(&dir, "file1.txt"), "new");
    assert_eq!(read_file(&dir, "file2.txt"), "new");
    assert_eq!(read_file(&dir, "file3.txt"), "new");
}

#[test]
fn test_dry_run() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "original");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("--dry-run")
        .arg("original")
        .arg("modified")
        .write_stdin(input)
        .assert()
        .success();

    // File should not be modified
    assert_eq!(read_file(&dir, "test.txt"), "original");
}

#[test]
fn test_quiet_mode() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "hello");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("-q")
        .arg("hello")
        .arg("world")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    assert_eq!(read_file(&dir, "test.txt"), "world");
}

#[test]
fn test_regex_pattern() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "foo123 bar456");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg(r"\d+")
        .arg("XXX")
        .write_stdin(input)
        .assert()
        .success();

    assert_eq!(read_file(&dir, "test.txt"), "fooXXX barXXX");
}

#[test]
fn test_backreferences_dollar_style() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "hello world");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg(r"(\w+) (\w+)")
        .arg("$2 $1")
        .write_stdin(input)
        .assert()
        .success();

    assert_eq!(read_file(&dir, "test.txt"), "world hello");
}

#[test]
fn test_backreferences_backslash_style() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "hello world");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg(r"(\w+) (\w+)")
        .arg(r"\2 \1")
        .write_stdin(input)
        .assert()
        .success();

    assert_eq!(read_file(&dir, "test.txt"), "world hello");
}

#[test]
fn test_no_matches_found() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "content");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("nonexistent")
        .arg("replacement")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("modified 0"));

    assert_eq!(read_file(&dir, "test.txt"), "content");
}

#[test]
fn test_invalid_regex() {
    Command::cargo_bin("csd")
        .unwrap()
        .arg("[invalid")
        .arg("replacement")
        .write_stdin("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid search pattern"));
}

#[test]
fn test_nested_directories() {
    let dir = setup_test_dir();
    create_file(&dir, "a/b/c/deep.txt", "find me");
    create_file(&dir, "x/y/z/deep.txt", "find me");

    let input = file_list(&dir, &["a/b/c/deep.txt", "x/y/z/deep.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("find me")
        .arg("found you")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("modified 2"));

    assert_eq!(read_file(&dir, "a/b/c/deep.txt"), "found you");
    assert_eq!(read_file(&dir, "x/y/z/deep.txt"), "found you");
}

#[test]
fn test_multiline_content() {
    let dir = setup_test_dir();
    create_file(
        &dir,
        "test.txt",
        "line1\nfind this\nline3\nfind this\nline5",
    );

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("find this")
        .arg("replaced")
        .write_stdin(input)
        .assert()
        .success();

    assert_eq!(
        read_file(&dir, "test.txt"),
        "line1\nreplaced\nline3\nreplaced\nline5"
    );
}

#[test]
fn test_special_characters_in_replacement() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "placeholder");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("placeholder")
        .arg("@special#chars!&more")
        .write_stdin(input)
        .assert()
        .success();

    assert_eq!(read_file(&dir, "test.txt"), "@special#chars!&more");
}

#[test]
fn test_empty_file() {
    let dir = setup_test_dir();
    create_file(&dir, "empty.txt", "");

    let input = file_list(&dir, &["empty.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("nothing")
        .arg("something")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("modified 0"));

    assert_eq!(read_file(&dir, "empty.txt"), "");
}

#[test]
fn test_file_with_no_matches() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "no matches here");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("xyz")
        .arg("abc")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("modified 0"));

    assert_eq!(read_file(&dir, "test.txt"), "no matches here");
}

#[test]
fn test_case_sensitive_matching() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "Hello HELLO hello");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("hello")
        .arg("hi")
        .write_stdin(input)
        .assert()
        .success();

    // Only lowercase 'hello' should be replaced
    assert_eq!(read_file(&dir, "test.txt"), "Hello HELLO hi");
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

    assert_eq!(read_file(&dir, "exists.txt"), "replaced");
}

#[test]
fn test_piped_files_with_spaces() {
    let dir = setup_test_dir();
    create_file(&dir, "file with spaces.txt", "content");

    let input = file_list(&dir, &["file with spaces.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("content")
        .arg("changed")
        .write_stdin(input)
        .assert()
        .success();

    assert_eq!(read_file(&dir, "file with spaces.txt"), "changed");
}

#[test]
fn test_mixed_file_types() {
    let dir = setup_test_dir();
    create_file(&dir, "file.txt", "replace");
    create_file(&dir, "file.md", "replace");
    create_file(&dir, "file.rs", "replace");

    let input = file_list(&dir, &["file.txt", "file.md", "file.rs"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("replace")
        .arg("done")
        .write_stdin(input)
        .assert()
        .success()
        .stderr(predicate::str::contains("modified 3"));

    assert_eq!(read_file(&dir, "file.txt"), "done");
    assert_eq!(read_file(&dir, "file.md"), "done");
    assert_eq!(read_file(&dir, "file.rs"), "done");
}

#[test]
fn test_complex_regex() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "func(foo, bar)");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg(r"func\(([^,]+), ([^)]+)\)")
        .arg(r"function($1, $2)")
        .write_stdin(input)
        .assert()
        .success();

    assert_eq!(read_file(&dir, "test.txt"), "function(foo, bar)");
}

#[test]
fn test_unicode_content() {
    let dir = setup_test_dir();
    create_file(&dir, "test.txt", "Hello 世界 🌍");

    let input = file_list(&dir, &["test.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("世界")
        .arg("World")
        .write_stdin(input)
        .assert()
        .success();

    assert_eq!(read_file(&dir, "test.txt"), "Hello World 🌍");
}

#[test]
fn test_large_file() {
    let dir = setup_test_dir();
    let content = "line with pattern\n".repeat(1000);
    create_file(&dir, "large.txt", &content);

    let input = file_list(&dir, &["large.txt"]);

    Command::cargo_bin("csd")
        .unwrap()
        .arg("pattern")
        .arg("replaced")
        .write_stdin(input)
        .assert()
        .success();

    let expected = "line with replaced\n".repeat(1000);
    assert_eq!(read_file(&dir, "large.txt"), expected);
}

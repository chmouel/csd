use std::io::{self, BufRead};
use std::path::PathBuf;

use ignore::WalkBuilder;
use regex::Regex;

/// Check if a file is likely binary by looking for null bytes in the first 8KB.
pub fn is_binary(path: &std::path::Path) -> bool {
    use std::io::Read;
    let Ok(mut f) = std::fs::File::open(path) else {
        return true;
    };
    let mut buf = [0u8; 8192];
    let Ok(n) = f.read(&mut buf) else {
        return true;
    };
    buf[..n].contains(&0)
}

/// Walk the current directory for files matching an optional file pattern regex.
/// Respects .gitignore and .ignore files unless `no_ignore` is set.
pub fn walk_files(file_pattern: Option<&Regex>, no_ignore: bool, include_git_dir: bool) -> Vec<PathBuf> {
    let mut builder = WalkBuilder::new(".");
    builder
        .hidden(false) // don't skip hidden files by default (fd behavior)
        .git_ignore(!no_ignore)
        .git_global(!no_ignore)
        .git_exclude(!no_ignore)
        .ignore(!no_ignore);

    let mut files = Vec::new();

    for entry in builder.build() {
        let Ok(entry) = entry else { continue };
        // Skip anything inside .git directory unless explicitly included
        if !include_git_dir
            && entry
                .path()
                .components()
                .any(|c| c.as_os_str() == ".git")
        {
            continue;
        }
        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }
        let path = entry.into_path();

        // Filter by file pattern if provided
        if let Some(pattern) = file_pattern {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let path_str = path.to_str().unwrap_or("");
            if !pattern.is_match(file_name) && !pattern.is_match(path_str) {
                continue;
            }
        }

        // Skip binary files
        if is_binary(&path) {
            continue;
        }

        files.push(path);
    }

    files
}

/// Read file paths from stdin (one per line).
pub fn read_stdin_files() -> Vec<PathBuf> {
    let stdin = io::stdin();
    let mut files = Vec::new();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { continue };
        let line = line.trim().to_string();
        if !line.is_empty() {
            let path = PathBuf::from(line);
            if path.is_file() && !is_binary(&path) {
                files.push(path);
            }
        }
    }
    files
}

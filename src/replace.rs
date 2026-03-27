use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use regex::Regex;

use crate::diff;

const STREAMING_THRESHOLD: u64 = 1024 * 1024; // 1MB

pub struct ReplaceOptions {
    pub interactive: bool,
    pub dry_run: bool,
    pub quiet: bool,
}

/// Process a single file: search and replace, returning true if modified.
pub fn process_file(
    path: &Path,
    search_regex: &Regex,
    replacement: &str,
    opts: &ReplaceOptions,
) -> Result<bool> {
    let metadata = fs::metadata(path).context("reading file metadata")?;
    let file_size = metadata.len();

    if file_size > STREAMING_THRESHOLD {
        process_file_streaming(path, search_regex, replacement, opts)
    } else {
        process_file_inmemory(path, search_regex, replacement, opts)
    }
}

/// In-memory processing for files under the streaming threshold.
fn process_file_inmemory(
    path: &Path,
    search_regex: &Regex,
    replacement: &str,
    opts: &ReplaceOptions,
) -> Result<bool> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;

    // Fast check: does the file even contain a match?
    if !search_regex.is_match(&content) {
        return Ok(false);
    }

    let new_content = if opts.interactive {
        interactive_replace(path, &content, search_regex, replacement)?
    } else {
        let replaced = search_regex.replace_all(&content, replacement);
        if replaced == content {
            return Ok(false);
        }
        replaced.into_owned()
    };

    if new_content == content {
        return Ok(false);
    }

    if !opts.quiet {
        if opts.dry_run {
            eprintln!("[Dry Run] Would modify: {}", path.display());
        } else {
            eprintln!("Modified: {}", path.display());
        }
    }

    if !opts.dry_run {
        atomic_write(path, new_content.as_bytes())?;
    }

    Ok(true)
}

/// Streaming processing for large files.
fn process_file_streaming(
    path: &Path,
    search_regex: &Regex,
    replacement: &str,
    opts: &ReplaceOptions,
) -> Result<bool> {
    // For large files, mmap for read, then process line by line
    let file = fs::File::open(path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let content = String::from_utf8_lossy(&mmap);

    if !search_regex.is_match(&content) {
        return Ok(false);
    }

    let new_content = if opts.interactive {
        interactive_replace(path, &content, search_regex, replacement)?
    } else {
        let replaced = search_regex.replace_all(&content, replacement);
        if replaced == *content {
            return Ok(false);
        }
        replaced.into_owned()
    };

    if new_content == *content {
        return Ok(false);
    }

    if !opts.quiet {
        if opts.dry_run {
            eprintln!("[Dry Run] Would modify: {}", path.display());
        } else {
            eprintln!("Modified: {}", path.display());
        }
    }

    if !opts.dry_run {
        // Drop mmap before writing
        drop(mmap);
        drop(file);
        atomic_write(path, new_content.as_bytes())?;
    }

    Ok(true)
}

/// Interactive replacement: iterate line by line, prompting for each match.
fn interactive_replace(
    path: &Path,
    content: &str,
    search_regex: &Regex,
    replacement: &str,
) -> Result<String> {
    // Count total matches for progress display
    let total_matches = search_regex.find_iter(content).count();
    let mut match_num = 0;

    let mut result = String::with_capacity(content.len());
    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        if !search_regex.is_match(line) {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Process matches in this line
        let mut current_line = line.to_string();
        let mut offset: i64 = 0;

        let matches: Vec<_> = search_regex.find_iter(line).collect();
        for m in &matches {
            match_num += 1;

            let adjusted_start = (m.start() as i64 + offset) as usize;
            let adjusted_end = (m.end() as i64 + offset) as usize;
            let match_text = &current_line[adjusted_start..adjusted_end];

            // Build the replacement text (expand capture groups)
            let replacement_text = search_regex.replace(match_text, replacement).into_owned();

            // Build the new line for display
            let new_line = format!(
                "{}{}{}",
                &current_line[..adjusted_start],
                &replacement_text,
                &current_line[adjusted_end..]
            );

            match diff::confirm_change(path, &current_line, &new_line, line_num, match_num, total_matches) {
                Ok(true) => {
                    let len_diff = replacement_text.len() as i64 - (adjusted_end - adjusted_start) as i64;
                    current_line = new_line;
                    offset += len_diff;
                }
                Ok(false) => {}
                Err(()) => {
                    // User pressed 'q' — return content as-is up to here
                    result.push_str(&current_line);
                    result.push('\n');
                    // Append remaining lines unchanged
                    for remaining_line in content.lines().skip(line_idx + 1) {
                        result.push_str(remaining_line);
                        result.push('\n');
                    }
                    return Ok(result);
                }
            }
        }

        result.push_str(&current_line);
        result.push('\n');
    }

    // Handle case where original content doesn't end with newline
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    Ok(result)
}

/// Atomically write to a file by writing to a temp file and renaming.
fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    use std::io::Write;

    // Preserve original permissions
    let metadata = fs::metadata(path).ok();

    let dir = path.parent().unwrap_or(Path::new("."));
    let file_name = path.file_name().unwrap_or_default();
    let tmp_path = dir.join(format!(".{}.csd.tmp", file_name.to_string_lossy()));

    let mut tmp_file = fs::File::create(&tmp_path)
        .with_context(|| format!("creating temp file {}", tmp_path.display()))?;
    tmp_file.write_all(content)?;
    tmp_file.flush()?;

    // Restore permissions
    if let Some(meta) = metadata {
        fs::set_permissions(&tmp_path, meta.permissions()).ok();
    }

    fs::rename(&tmp_path, path)
        .with_context(|| format!("renaming {} to {}", tmp_path.display(), path.display()))?;

    Ok(())
}

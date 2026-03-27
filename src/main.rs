mod diff;
mod replace;
mod walk;

use std::io::IsTerminal;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use regex::Regex;

use replace::ReplaceOptions;

/// A super-fast search-and-replace tool for files.
///
/// Performs regex-based search and replace operations across multiple files.
///
/// Examples:
///   csd '\.txt$' 'hello' 'world'        # Replace in .txt files
///   csd 'old_func' 'new_func'            # Replace in all files
///   echo file.txt | csd 'search' 'repl'  # Piped file list
#[derive(Parser)]
#[command(name = "csd", version, about)]
struct Cli {
    /// Patterns: [FILE_PATTERN] SEARCH_PATTERN REPLACEMENT_PATTERN
    #[arg(required = true, num_args = 2..=3)]
    patterns: Vec<String>,

    /// Prompt for confirmation for each change with a diff
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    /// Suppress all output except errors
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Don't respect .gitignore/.ignore files
    #[arg(short = 'I', long = "no-ignore")]
    no_ignore: bool,

    /// Include .git directory contents (also settable via CSD_INCLUDE_GIT_DIR=1)
    #[arg(long = "include-git-dir", env = "CSD_INCLUDE_GIT_DIR")]
    include_git_dir: bool,

    /// Show what would change without modifying files
    #[arg(long = "dry-run")]
    dry_run: bool,
}

/// Convert Python-style backreferences (\1, \2) to Rust regex style ($1, $2).
fn convert_backrefs(replacement: &str) -> String {
    let backref_re = Regex::new(r"\\(\d+)").unwrap();
    backref_re.replace_all(replacement, r"$$$1").into_owned()
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Only treat stdin as piped file list when:
    // 1. stdin is not a terminal AND
    // 2. exactly 2 positional args are given (search + replacement)
    // With 3 args, the first is always a file pattern for walking.
    let stdin_is_pipe = !std::io::stdin().is_terminal() && cli.patterns.len() == 2;

    let (file_pattern_str, search_pattern, replacement_raw) = if stdin_is_pipe {
        (None, cli.patterns[0].clone(), cli.patterns[1].clone())
    } else if cli.patterns.len() == 2 {
        (None, cli.patterns[0].clone(), cli.patterns[1].clone())
    } else {
        (
            Some(cli.patterns[0].clone()),
            cli.patterns[1].clone(),
            cli.patterns[2].clone(),
        )
    };

    // Auto-convert \1 style backrefs to $1
    let replacement = convert_backrefs(&replacement_raw);

    // Compile regexes
    let file_pattern = match &file_pattern_str {
        Some(pat) => Some(Regex::new(pat).map_err(|e| anyhow::anyhow!("Invalid file pattern: {}", e))?),
        None => None,
    };
    let search_regex =
        Regex::new(&search_pattern).map_err(|e| anyhow::anyhow!("Invalid search pattern: {}", e))?;

    // Collect files
    let files = if stdin_is_pipe {
        walk::read_stdin_files()
    } else {
        walk::walk_files(file_pattern.as_ref(), cli.no_ignore, cli.include_git_dir)
    };

    if files.is_empty() {
        if !cli.quiet {
            eprintln!("No matching files found.");
        }
        return Ok(());
    }

    let opts = ReplaceOptions {
        interactive: cli.interactive,
        dry_run: cli.dry_run,
        quiet: cli.quiet,
    };

    let modified_count = if cli.interactive {
        // Sequential for interactive mode
        let mut count = 0usize;
        for path in &files {
            match replace::process_file(path, &search_regex, &replacement, &opts) {
                Ok(true) => count += 1,
                Ok(false) => {}
                Err(e) => {
                    if !cli.quiet {
                        eprintln!("Error processing {}: {}", path.display(), e);
                    }
                }
            }
        }
        count
    } else {
        // Parallel with rayon
        let count = AtomicUsize::new(0);
        files.par_iter().for_each(|path| {
            match replace::process_file(path, &search_regex, &replacement, &opts) {
                Ok(true) => {
                    count.fetch_add(1, Ordering::Relaxed);
                }
                Ok(false) => {}
                Err(e) => {
                    if !cli.quiet {
                        eprintln!("Error processing {}: {}", path.display(), e);
                    }
                }
            }
        });
        count.load(Ordering::Relaxed)
    };

    if !cli.quiet {
        eprintln!(
            "\nDone. Processed {} files, modified {}.",
            files.len(),
            modified_count
        );
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}

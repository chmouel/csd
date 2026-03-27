use std::io::{self, BufRead, Write};
use std::path::Path;

const RED: &str = "\x1b[91m";
const GREEN: &str = "\x1b[92m";
const RESET: &str = "\x1b[0m";

/// Display a colored diff of a proposed change and prompt the user.
/// Returns Ok(true) to apply, Ok(false) to skip, Err on quit/cancel.
pub fn confirm_change(
    filename: &Path,
    old_line: &str,
    new_line: &str,
    line_num: usize,
    match_num: usize,
    total_matches: usize,
) -> Result<bool, ()> {
    let stderr = io::stderr();
    let mut out = stderr.lock();

    writeln!(
        out,
        "\n--- Change {}/{} in {} (Line {}) ---",
        match_num,
        total_matches,
        filename.display(),
        line_num
    )
    .ok();

    // Show old line in red, new line in green
    writeln!(out, "{RED}-{}{RESET}", old_line.trim_end()).ok();
    writeln!(out, "{GREEN}+{}{RESET}", new_line.trim_end()).ok();

    write!(out, "Apply this change? [Y/n/q] ").ok();
    out.flush().ok();

    // Read from /dev/tty so it works even when stdin is piped
    let response = read_tty_line();

    match response.trim().to_lowercase().as_str() {
        "q" => Err(()),
        "n" | "no" => Ok(false),
        "" | "y" | "yes" => Ok(true),
        _ => Ok(false),
    }
}

/// Read a line from /dev/tty (falls back to stdin).
fn read_tty_line() -> String {
    // Try /dev/tty first (works on macOS/Linux even when stdin is piped)
    if let Ok(tty) = std::fs::File::open("/dev/tty") {
        let mut reader = io::BufReader::new(tty);
        let mut line = String::new();
        if reader.read_line(&mut line).is_ok() {
            return line;
        }
    }

    // Fallback to stdin
    let mut line = String::new();
    io::stdin().read_line(&mut line).ok();
    line
}

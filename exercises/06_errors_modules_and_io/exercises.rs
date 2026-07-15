//! Exercises for module 6: errors, `Result`, and file I/O.
//!
//! Implement each `todo!()` body, then run the example tests. Do not change any
//! signature. Propagate I/O errors with `?` rather than panicking.

use std::io;
use std::path::Path;

/// Parse `raw` (after trimming) as a positive integer.
///
/// Returns `Err` with a descriptive message when the input is not a number or is
/// zero; only strictly positive values are accepted.
pub fn parse_positive(_raw: &str) -> Result<u32, String> {
    todo!("parse, reject zero, and preserve a useful message")
}

/// Read the UTF-8 contents of `path` and return them trimmed.
///
/// Propagates any I/O error (for example, a missing file) to the caller.
pub fn read_trimmed(_path: &Path) -> io::Result<String> {
    todo!("read UTF-8 text and trim it")
}

/// Write each entry of `lines` to `path` as `"{n}. {line}\n"`, numbered from 1.
///
/// Propagates any I/O error to the caller.
pub fn write_numbered_lines(_path: &Path, _lines: &[&str]) -> io::Result<()> {
    todo!("write one 1-based numbered line per input")
}

fn main() {
    println!("Run `cargo test --example ex-06-errors-io` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_only_positive_integers() {
        assert_eq!(parse_positive(" 42 "), Ok(42));
        assert!(parse_positive("0").is_err());
        assert!(parse_positive("many").is_err());
    }

    #[test]
    fn reads_and_writes_files() -> io::Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("notes.txt");
        write_numbered_lines(&path, &["ownership", "borrowing"])?;
        assert_eq!(fs::read_to_string(&path)?, "1. ownership\n2. borrowing\n");
        assert_eq!(read_trimmed(&path)?, "1. ownership\n2. borrowing");
        Ok(())
    }

    #[test]
    fn propagates_missing_file_error() {
        let result = read_trimmed(Path::new("path-that-does-not-exist.txt"));
        assert!(result.is_err());
    }
}

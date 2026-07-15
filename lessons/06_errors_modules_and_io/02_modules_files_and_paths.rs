//! Lesson 6.2: module privacy, paths, buffered file I/O, and RAII.
//!
//! Items are private to their module unless marked `pub` (or `pub(crate)` for
//! crate-wide visibility). File handles follow RAII: they are closed
//! automatically when dropped. Buffered readers cut down on syscalls, and
//! `io::Result` with `?` propagates I/O errors rather than ignoring them.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

mod units {
    pub const METERS_PER_KILOMETER: f64 = 1_000.0;

    // `pub` makes this callable from outside the module.
    pub fn kilometers_to_meters(kilometers: f64) -> f64 {
        kilometers * METERS_PER_KILOMETER
    }

    // `pub(crate)` restricts visibility to this crate; external crates cannot see
    // it even though it is `pub`-ish within the project.
    pub(crate) fn course_example() -> &'static str {
        "visible within this crate"
    }
}

fn write_lines(path: &Path, lines: &[&str]) -> io::Result<()> {
    // `File::create` truncates or creates the file; the handle is closed on drop.
    let mut file = File::create(path)?;
    for line in lines {
        writeln!(file, "{line}")?; // `?` bubbles any write error up to the caller
    }
    file.flush()
}

fn read_non_empty_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    // `BufReader` batches reads so `lines()` does not hit the OS per line.
    let reader = BufReader::new(file);
    reader
        .lines()
        // Keep every read error, but drop lines that are blank after trimming.
        .filter_map(|line| match line {
            Ok(value) if value.trim().is_empty() => None,
            other => Some(other),
        })
        .collect()
}

fn main() -> io::Result<()> {
    println!("2.5 km = {} m", units::kilometers_to_meters(2.5));
    println!("{}", units::course_example());

    // TempDir creates a unique directory and removes it automatically on drop.
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("notes.txt");
    write_lines(&path, &["ownership", "", "borrowing", "results"])?;
    let lines = read_non_empty_lines(&path)?;
    println!("read from {}: {lines:?}", path.display());
    Ok(())
}

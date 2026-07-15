//! Lesson 6.2: module privacy, paths, buffered file I/O, and RAII.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

mod units {
    pub const METERS_PER_KILOMETER: f64 = 1_000.0;

    pub fn kilometers_to_meters(kilometers: f64) -> f64 {
        kilometers * METERS_PER_KILOMETER
    }

    pub(crate) fn course_example() -> &'static str {
        "visible within this crate"
    }
}

fn write_lines(path: &Path, lines: &[&str]) -> io::Result<()> {
    let mut file = File::create(path)?;
    for line in lines {
        writeln!(file, "{line}")?;
    }
    file.flush()
}

fn read_non_empty_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader
        .lines()
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

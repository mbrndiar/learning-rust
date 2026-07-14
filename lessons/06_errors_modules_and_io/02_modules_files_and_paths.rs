//! Lesson 6.2: module privacy, paths, buffered file I/O, and RAII.

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

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

fn temporary_path() -> PathBuf {
    std::env::temp_dir().join(format!("learning-rust-{}.txt", std::process::id()))
}

fn main() -> io::Result<()> {
    println!("2.5 km = {} m", units::kilometers_to_meters(2.5));
    println!("{}", units::course_example());

    let path = temporary_path();
    write_lines(&path, &["ownership", "", "borrowing", "results"])?;
    let lines = read_non_empty_lines(&path)?;
    println!("read from {}: {lines:?}", path.display());
    fs::remove_file(path)?;
    Ok(())
}

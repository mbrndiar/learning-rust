//! Reference solutions for module 6.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

fn parse_positive(raw: &str) -> Result<u32, String> {
    let value: u32 = raw
        .trim()
        .parse()
        .map_err(|error| format!("expected a positive integer: {error}"))?;
    if value == 0 {
        Err(String::from("value must be greater than zero"))
    } else {
        Ok(value)
    }
}

fn read_trimmed(path: &Path) -> io::Result<String> {
    Ok(fs::read_to_string(path)?.trim().to_owned())
}

fn write_numbered_lines(path: &Path, lines: &[&str]) -> io::Result<()> {
    let mut file = File::create(path)?;
    for (index, line) in lines.iter().enumerate() {
        writeln!(file, "{}. {line}", index + 1)?;
    }
    file.flush()
}

fn main() -> io::Result<()> {
    assert_eq!(parse_positive(" 42 "), Ok(42));
    assert!(parse_positive("0").is_err());

    let directory = tempfile::tempdir()?;
    let path = directory.path().join("notes.txt");
    write_numbered_lines(&path, &["ownership", "borrowing"])?;
    assert_eq!(read_trimmed(&path)?, "1. ownership\n2. borrowing");
    println!("Module 6 solutions passed.");
    Ok(())
}

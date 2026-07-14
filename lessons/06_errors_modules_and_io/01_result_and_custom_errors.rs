//! Lesson 6.1: `Result`, `?`, custom errors, and error conversion.

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    InvalidNumber(ParseIntError),
    PortOutOfRange(u32),
    EmptyHost,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumber(_) => write!(formatter, "port must be a whole number"),
            Self::PortOutOfRange(port) => write!(formatter, "port {port} is outside 1..=65535"),
            Self::EmptyHost => write!(formatter, "host must not be empty"),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidNumber(source) => Some(source),
            Self::PortOutOfRange(_) | Self::EmptyHost => None,
        }
    }
}

impl From<ParseIntError> for ConfigError {
    fn from(error: ParseIntError) -> Self {
        Self::InvalidNumber(error)
    }
}

fn parse_address(host: &str, raw_port: &str) -> Result<String, ConfigError> {
    if host.trim().is_empty() {
        return Err(ConfigError::EmptyHost);
    }

    let port: u32 = raw_port.parse()?;
    if !(1..=u32::from(u16::MAX)).contains(&port) {
        return Err(ConfigError::PortOutOfRange(port));
    }

    Ok(format!("{}:{port}", host.trim()))
}

fn main() {
    for (host, port) in [("127.0.0.1", "8080"), ("", "80"), ("localhost", "99999")] {
        match parse_address(host, port) {
            Ok(address) => println!("valid address: {address}"),
            Err(error) => {
                eprintln!("configuration error: {error}");
                if let Some(source) = error.source() {
                    eprintln!("  caused by: {source}");
                }
            }
        }
    }
}

//! Reference solutions for module 10.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ServerInput {
    host: String,
    port: u16,
    workers: Option<u8>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: u8,
}

fn validate(input: ServerInput) -> Result<ServerConfig, String> {
    let host = input.host.trim();
    if host.is_empty() {
        return Err(String::from("host must not be empty"));
    }
    if input.port == 0 {
        return Err(String::from("port must be nonzero"));
    }
    let workers = input.workers.unwrap_or(1);
    if workers == 0 {
        return Err(String::from("workers must be nonzero"));
    }
    Ok(ServerConfig {
        host: host.to_owned(),
        port: input.port,
        workers,
    })
}

fn decode_config(json: &str) -> Result<ServerConfig, String> {
    let input: ServerInput =
        serde_json::from_str(json).map_err(|error| format!("invalid JSON: {error}"))?;
    validate(input)
}

fn main() {
    let config = decode_config(r#"{"host":" localhost ","port":8080}"#).expect("valid config");
    assert_eq!(
        config,
        ServerConfig {
            host: String::from("localhost"),
            port: 8080,
            workers: 1,
        }
    );
    assert!(decode_config("{not json}").is_err());
    println!("Module 10 solutions passed.");
}

//! Exercises for module 10.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ServerInput {
    pub host: String,
    pub port: u16,
    pub workers: Option<u8>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: u8,
}

pub fn validate(_input: ServerInput) -> Result<ServerConfig, String> {
    todo!("validate host, port, and worker count")
}

pub fn decode_config(_json: &str) -> Result<ServerConfig, String> {
    todo!("deserialize first, then validate")
}

fn main() {
    println!("Run `cargo test --example ex-10-integration` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_and_defaults_valid_input() {
        assert_eq!(
            decode_config(r#"{"host":" localhost ","port":8080}"#),
            Ok(ServerConfig {
                host: String::from("localhost"),
                port: 8080,
                workers: 1,
            })
        );
    }

    #[test]
    fn separates_decode_and_validation_failures() {
        assert!(decode_config("{not json}").unwrap_err().contains("JSON"));
        assert!(
            decode_config(r#"{"host":"","port":8080}"#)
                .unwrap_err()
                .contains("host")
        );
        assert!(
            decode_config(r#"{"host":"localhost","port":0,"workers":2}"#)
                .unwrap_err()
                .contains("port")
        );
        assert!(
            decode_config(r#"{"host":"localhost","port":8080,"workers":0}"#)
                .unwrap_err()
                .contains("workers")
        );
    }

    #[test]
    fn serializes_the_validated_shape() {
        let config = decode_config(r#"{"host":"localhost","port":8080}"#).expect("valid");
        assert_eq!(
            serde_json::to_string(&config).expect("serializable"),
            r#"{"host":"localhost","port":8080,"workers":1}"#
        );
    }
}

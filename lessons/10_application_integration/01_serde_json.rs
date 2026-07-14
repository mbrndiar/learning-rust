//! Lesson 10.1: deserialize a wire type, then validate a domain type.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UserInput {
    name: String,
    age: u8,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq)]
struct User {
    name: String,
    age: u8,
    tags: Vec<String>,
}

#[derive(Debug, PartialEq)]
enum ValidationError {
    EmptyName,
    EmptyTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(formatter, "name must not be empty"),
            Self::EmptyTag => write!(formatter, "tags must not be empty"),
        }
    }
}

impl Error for ValidationError {}

impl TryFrom<UserInput> for User {
    type Error = ValidationError;

    fn try_from(input: UserInput) -> Result<Self, Self::Error> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(ValidationError::EmptyName);
        }

        let tags: Vec<String> = input
            .tags
            .into_iter()
            .map(|tag| tag.trim().to_lowercase())
            .collect();
        if tags.iter().any(String::is_empty) {
            return Err(ValidationError::EmptyTag);
        }

        Ok(Self {
            name: name.to_owned(),
            age: input.age,
            tags,
        })
    }
}

fn decode_user(json: &str) -> Result<User, Box<dyn Error>> {
    let input: UserInput = serde_json::from_str(json)?;
    Ok(User::try_from(input)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let json = r#"{"name":" Ada ","age":36,"tags":["Rust"," Teacher "]}"#;
    let user = decode_user(json)?;
    println!("validated user={user:?}");
    println!("encoded JSON={}", serde_json::to_string_pretty(&user)?);

    for invalid in [
        r#"{"name":" ","age":20}"#,
        r#"{"name":"Ada","age":20,"unexpected":true}"#,
    ] {
        println!("decode {invalid}: {:?}", decode_user(invalid));
    }
    Ok(())
}

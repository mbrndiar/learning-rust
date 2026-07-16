//! Lesson 13.1: strict JSON wire types and validated domain conversion.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateLabelRequest {
    name: String,
    color: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct Label {
    name: String,
    color: String,
}

#[derive(Debug, PartialEq, Eq)]
enum ValidationError {
    EmptyName,
    InvalidColor,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(formatter, "name must not be empty"),
            Self::InvalidColor => write!(formatter, "color must be a # followed by six hex digits"),
        }
    }
}

impl Error for ValidationError {}

impl TryFrom<CreateLabelRequest> for Label {
    type Error = ValidationError;

    fn try_from(request: CreateLabelRequest) -> Result<Self, Self::Error> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(ValidationError::EmptyName);
        }
        if request.color.len() != 7
            || !request.color.starts_with('#')
            || !request.color[1..]
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err(ValidationError::InvalidColor);
        }
        Ok(Self {
            name: name.to_owned(),
            color: request.color.to_ascii_lowercase(),
        })
    }
}

fn decode_label(json: &str) -> Result<Label, Box<dyn Error>> {
    let request: CreateLabelRequest = serde_json::from_str(json)?;
    Ok(Label::try_from(request)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let label = decode_label(r##"{"name":" Rust ","color":"#FF6600"}"##)?;
    assert_eq!(label.name, "Rust");

    let syntax_error = decode_label(r#"{"name": "#);
    let validation_error = decode_label(r##"{"name":" ","color":"#ff6600"}"##);
    let unknown_field = decode_label(r##"{"name":"Rust","color":"#ff6600","owner":"Ada"}"##);

    println!("valid={}", serde_json::to_string(&label)?);
    println!("syntax/type error={syntax_error:?}");
    println!("validation error={validation_error:?}");
    println!("unknown field error={unknown_field:?}");
    Ok(())
}

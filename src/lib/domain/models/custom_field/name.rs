use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvalidName {
    #[error("cannot be empty")]
    Empty,
    #[error("cannot be longer than 64 characters")]
    TooLong,
    #[error("cannot contain special characters")]
    InvalidCharacters,
    #[error("cannot contain whitespace")]
    InvalidWhitespace,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Name(String);

impl Name {
    pub fn new(name: &str) -> Result<Self, InvalidName> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            Err(InvalidName::Empty)
        } else if trimmed.len() > 64 {
            Err(InvalidName::TooLong)
        } else if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '_') {
            Err(InvalidName::InvalidCharacters)
        } else if trimmed.contains(|c: char| c.is_whitespace()) {
            Err(InvalidName::InvalidWhitespace)
        } else {
            Ok(Name(trimmed.to_string()))
        }
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

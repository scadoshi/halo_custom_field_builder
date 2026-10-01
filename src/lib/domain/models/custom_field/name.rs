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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_letters_digits_and_underscores_up_to_64_trimmed() {
        assert_eq!(
            Name::new(" pizza_Size2 ").unwrap().to_string(),
            "pizza_Size2"
        );
        assert!(Name::new(&"x".repeat(64)).is_ok());
        assert!(matches!(
            Name::new(&"x".repeat(65)),
            Err(InvalidName::TooLong)
        ));
        assert!(matches!(Name::new(""), Err(InvalidName::Empty)));
        assert!(matches!(
            Name::new("pizza-size"),
            Err(InvalidName::InvalidCharacters)
        ));
        // A space inside is caught by the character rule before the whitespace rule.
        assert!(matches!(
            Name::new("pizza size"),
            Err(InvalidName::InvalidCharacters)
        ));
    }
}

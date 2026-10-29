//! The field's internal name.

use std::fmt::Display;

use thiserror::Error;

/// The most characters Halo takes in a field name.
const MAX_LEN: usize = 64;

/// Why a name is invalid.
#[derive(Debug, Error)]
pub enum ParseNameError {
    /// Nothing is left once trimmed.
    #[error("cannot be empty")]
    Empty,
    /// Too many characters once trimmed.
    #[error("cannot be longer than {} characters", MAX_LEN)]
    TooLong,
    /// A character other than an ASCII letter, digit or underscore.
    #[error("can only hold ASCII letters, digits and underscores")]
    InvalidCharacters,
}

/// A trimmed name of 1 to 64 ASCII letters, digits and underscores.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl Name {
    /// The name, trimmed.
    ///
    /// # Errors
    ///
    /// Fails when the trimmed name is empty, longer than 64 characters, or
    /// holds anything but ASCII letters, digits and underscores.
    pub fn new(name: &str) -> Result<Self, ParseNameError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            Err(ParseNameError::Empty)
        } else if trimmed.chars().count() > MAX_LEN {
            Err(ParseNameError::TooLong)
        } else if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            Err(ParseNameError::InvalidCharacters)
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
            Err(ParseNameError::TooLong)
        ));
        assert!(matches!(Name::new(""), Err(ParseNameError::Empty)));
        assert!(matches!(
            Name::new("pizza-size"),
            Err(ParseNameError::InvalidCharacters)
        ));
        assert!(matches!(
            Name::new("pizza size"),
            Err(ParseNameError::InvalidCharacters)
        ));
    }

    #[test]
    fn a_name_is_ascii_letters_digits_and_underscores_only() {
        for name in ["pizzá", "名前", "größe", "x\u{200b}y", &"é".repeat(64)] {
            assert!(
                matches!(Name::new(name), Err(ParseNameError::InvalidCharacters)),
                "{name:?}"
            );
        }
    }
}

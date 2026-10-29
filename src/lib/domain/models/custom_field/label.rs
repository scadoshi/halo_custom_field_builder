use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseLabelError {
    #[error("cannot be empty")]
    Empty,
    #[error("cannot be longer than 256 characters")]
    TooLong,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(String);

impl Label {
    pub fn new(label: &str) -> Result<Self, ParseLabelError> {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            Err(ParseLabelError::Empty)
        } else if trimmed.chars().count() > 256 {
            Err(ParseLabelError::TooLong)
        } else {
            Ok(Self(trimmed.to_string()))
        }
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_label_is_anything_visible_up_to_256_characters() {
        assert!(Label::new("pizza size").is_ok());
        assert!(matches!(Label::new(""), Err(ParseLabelError::Empty)));
        assert!(matches!(Label::new("   "), Err(ParseLabelError::Empty)));
        assert!(Label::new(&"x".repeat(256)).is_ok());
        assert!(matches!(
            Label::new(&"x".repeat(257)),
            Err(ParseLabelError::TooLong)
        ));
    }

    #[test]
    fn a_label_is_stored_trimmed_and_measured_in_characters() {
        assert_eq!(
            Label::new(" pizza size ").unwrap().to_string(),
            "pizza size"
        );
        assert!(Label::new(&"é".repeat(256)).is_ok());
        assert!(matches!(
            Label::new(&"é".repeat(257)),
            Err(ParseLabelError::TooLong)
        ));
    }
}

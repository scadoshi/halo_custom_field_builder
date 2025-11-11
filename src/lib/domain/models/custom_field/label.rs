use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvalidLabel {
    #[error("cannot be empty")]
    Empty,
    #[error("cannot be longer than 256 characters")]
    TooLong,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Label(String);

impl Label {
    pub fn new(label: &str) -> Result<Self, InvalidLabel> {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            Err(InvalidLabel::Empty)
        } else if trimmed.len() > 256 {
            Err(InvalidLabel::TooLong)
        } else {
            Ok(Self(label.to_string()))
        }
    }

    pub fn id(&self) -> u8 {
        self.0.as_bytes()[0]
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

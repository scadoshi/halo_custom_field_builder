use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid text input type was given")]
pub struct InvalidTextInputType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextInputType {
    Anything,
    Integer,
    Money,
    Alphanumeric,
    Decimal,
    URL,
    Password,
}

impl TextInputType {
    pub fn input_type_id(&self) -> u8 {
        match self {
            TextInputType::Anything => 0,
            TextInputType::Integer => 1,
            TextInputType::Money => 2,
            TextInputType::Alphanumeric => 3,
            TextInputType::Decimal => 4,
            TextInputType::URL => 5,
            TextInputType::Password => 6,
        }
    }
}

impl TryFrom<u8> for TextInputType {
    type Error = InvalidTextInputType;
    fn try_from(input_type_id: u8) -> Result<Self, Self::Error> {
        match input_type_id {
            0 => Ok(TextInputType::Anything),
            1 => Ok(TextInputType::Integer),
            2 => Ok(TextInputType::Money),
            3 => Ok(TextInputType::Alphanumeric),
            4 => Ok(TextInputType::Decimal),
            5 => Ok(TextInputType::URL),
            6 => Ok(TextInputType::Password),
            _ => Err(InvalidTextInputType),
        }
    }
}

impl Serialize for TextInputType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.input_type_id())
    }
}

impl<'de> Deserialize<'de> for TextInputType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input_type_id = u8::deserialize(deserializer)?;
        Self::try_from(input_type_id).map_err(serde::de::Error::custom)
    }
}

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid date input type was given")]
pub struct InvalidDateInputType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DateInputType {
    Date,
    DateTime,
}

impl DateInputType {
    pub fn input_type_id(&self) -> u8 {
        match self {
            DateInputType::Date => 0,
            DateInputType::DateTime => 1,
        }
    }
}

impl TryFrom<u8> for DateInputType {
    type Error = InvalidDateInputType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DateInputType::Date),
            1 => Ok(DateInputType::DateTime),
            _ => Err(InvalidDateInputType),
        }
    }
}

impl Serialize for DateInputType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.input_type_id())
    }
}

impl<'de> Deserialize<'de> for DateInputType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input_type_id = u8::deserialize(deserializer)?;
        Self::try_from(input_type_id).map_err(serde::de::Error::custom)
    }
}

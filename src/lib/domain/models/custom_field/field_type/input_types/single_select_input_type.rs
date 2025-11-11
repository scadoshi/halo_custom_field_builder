use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid single select input type was given")]
pub struct InvalidSingleSelectInputType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SingleSelectInputType {
    Standard,
    Tree,
    Radio,
}

impl SingleSelectInputType {
    pub fn input_type_id(&self) -> u8 {
        match self {
            SingleSelectInputType::Standard => 0,
            SingleSelectInputType::Tree => 1,
            SingleSelectInputType::Radio => 2,
        }
    }
}

impl TryFrom<u8> for SingleSelectInputType {
    type Error = InvalidSingleSelectInputType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SingleSelectInputType::Standard),
            1 => Ok(SingleSelectInputType::Tree),
            2 => Ok(SingleSelectInputType::Radio),
            _ => Err(InvalidSingleSelectInputType),
        }
    }
}

impl Serialize for SingleSelectInputType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.input_type_id())
    }
}

impl<'de> Deserialize<'de> for SingleSelectInputType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input_type_id = u8::deserialize(deserializer)?;
        Self::try_from(input_type_id).map_err(serde::de::Error::custom)
    }
}

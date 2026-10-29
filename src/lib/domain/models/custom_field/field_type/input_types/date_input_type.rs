//! Input types of a date field, `field_type_id` 4.

use crate::domain::models::custom_field::field_type::ParseFieldTypeError;

/// How a date field is entered.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DateInputType {
    /// A date alone, `input_type_id` 0.
    Date,
    /// A date and time, `input_type_id` 1.
    DateTime,
}

impl DateInputType {
    /// The id Halo uses for this input type.
    pub fn input_type_id(&self) -> u8 {
        match self {
            DateInputType::Date => 0,
            DateInputType::DateTime => 1,
        }
    }
}

impl TryFrom<u8> for DateInputType {
    type Error = ParseFieldTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DateInputType::Date),
            1 => Ok(DateInputType::DateTime),
            _ => Err(ParseFieldTypeError::InvalidInputType {
                field_type_id: 4,
                input_type_id: value,
            }),
        }
    }
}

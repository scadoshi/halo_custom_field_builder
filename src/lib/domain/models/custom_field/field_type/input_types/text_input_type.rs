//! Input types of a text field, `field_type_id` 0.

use crate::domain::models::custom_field::field_type::ParseFieldTypeError;

/// What a text field may hold.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextInputType {
    /// Any text, `input_type_id` 0.
    Anything,
    /// A whole number, `input_type_id` 1.
    Integer,
    /// An amount of money, `input_type_id` 2.
    Money,
    /// Letters and digits, `input_type_id` 3.
    Alphanumeric,
    /// A decimal number, `input_type_id` 4.
    Decimal,
    /// A URL, `input_type_id` 5.
    Url,
    /// A masked password, `input_type_id` 6.
    Password,
}

impl TextInputType {
    /// The id Halo uses for this input type.
    pub fn input_type_id(&self) -> u8 {
        match self {
            TextInputType::Anything => 0,
            TextInputType::Integer => 1,
            TextInputType::Money => 2,
            TextInputType::Alphanumeric => 3,
            TextInputType::Decimal => 4,
            TextInputType::Url => 5,
            TextInputType::Password => 6,
        }
    }
}

impl TryFrom<u8> for TextInputType {
    type Error = ParseFieldTypeError;
    fn try_from(input_type_id: u8) -> Result<Self, Self::Error> {
        match input_type_id {
            0 => Ok(TextInputType::Anything),
            1 => Ok(TextInputType::Integer),
            2 => Ok(TextInputType::Money),
            3 => Ok(TextInputType::Alphanumeric),
            4 => Ok(TextInputType::Decimal),
            5 => Ok(TextInputType::Url),
            6 => Ok(TextInputType::Password),
            _ => Err(ParseFieldTypeError::InvalidInputType {
                field_type_id: 0,
                input_type_id,
            }),
        }
    }
}

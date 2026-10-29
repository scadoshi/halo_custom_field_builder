use crate::domain::models::custom_field::field_type::ParseFieldTypeError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextInputType {
    Anything,
    Integer,
    Money,
    Alphanumeric,
    Decimal,
    Url,
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

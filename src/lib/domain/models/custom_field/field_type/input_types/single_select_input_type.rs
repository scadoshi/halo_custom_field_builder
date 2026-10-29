use crate::domain::models::custom_field::field_type::ParseFieldTypeError;

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
    type Error = ParseFieldTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SingleSelectInputType::Standard),
            1 => Ok(SingleSelectInputType::Tree),
            2 => Ok(SingleSelectInputType::Radio),
            _ => Err(ParseFieldTypeError::InvalidInputType {
                field_type_id: 2,
                input_type_id: value,
            }),
        }
    }
}

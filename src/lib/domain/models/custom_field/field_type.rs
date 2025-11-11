pub mod input_types;
use crate::domain::models::custom_field::field_type::input_types::{
    date_input_type::{DateInputType, InvalidDateInputType},
    single_select_input_type::{InvalidSingleSelectInputType, SingleSelectInputType},
    text_input_type::{InvalidTextInputType, TextInputType},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvalidFieldType {
    #[error("invalid field type id was given")]
    InvalidFieldTypeId,
    #[error(transparent)]
    InvalidInputType(anyhow::Error),
}

impl From<InvalidTextInputType> for InvalidFieldType {
    fn from(value: InvalidTextInputType) -> Self {
        Self::InvalidInputType(value.into())
    }
}

impl From<InvalidDateInputType> for InvalidFieldType {
    fn from(value: InvalidDateInputType) -> Self {
        Self::InvalidInputType(value.into())
    }
}

impl From<InvalidSingleSelectInputType> for InvalidFieldType {
    fn from(value: InvalidSingleSelectInputType) -> Self {
        Self::InvalidInputType(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldType {
    Text {
        input_type: TextInputType,
    },
    Memo,
    SingleSelect {
        input_type: SingleSelectInputType,
        selection_options: Vec<String>,
    },
    MultiSelect {
        selection_options: Vec<String>,
    },
    Date {
        input_type: DateInputType,
    },
    Time,
    Checkbox,
    Rich,
}
impl FieldType {
    pub fn new(
        field_type_id: u8,
        input_type_id: Option<u8>,
        selection_options: Vec<String>,
    ) -> Result<Self, InvalidFieldType> {
        match field_type_id {
            0 => {
                let input_type_id = input_type_id.unwrap_or(0);
                let input_type = TextInputType::try_from(input_type_id)?;
                Ok(Self::Text { input_type })
            }
            1 => Ok(FieldType::Memo),
            2 => {
                let input_type_id = input_type_id.unwrap_or(0);
                let input_type = SingleSelectInputType::try_from(input_type_id)?;
                Ok(Self::SingleSelect {
                    input_type,
                    selection_options,
                })
            }
            3 => Ok(FieldType::MultiSelect { selection_options }),
            4 => {
                let input_type_id = input_type_id.unwrap_or(0);
                let input_type = DateInputType::try_from(input_type_id)?;
                Ok(Self::Date { input_type })
            }
            5 => Ok(FieldType::Time),
            6 => Ok(FieldType::Checkbox),
            10 => Ok(FieldType::Rich),
            _ => Err(InvalidFieldType::InvalidFieldTypeId),
        }
    }

    pub fn input_type_id(&self) -> Option<u8> {
        match self {
            FieldType::Text { input_type } => Some(input_type.input_type_id()),
            FieldType::Memo => None,
            FieldType::SingleSelect { input_type, .. } => Some(input_type.input_type_id()),
            FieldType::MultiSelect { .. } => None,
            FieldType::Date { input_type } => Some(input_type.input_type_id()),
            FieldType::Time => None,
            FieldType::Checkbox => None,
            FieldType::Rich => None,
        }
    }

    pub fn selection_options(&self) -> Option<Vec<String>> {
        match self {
            FieldType::SingleSelect {
                selection_options, ..
            } => Some(selection_options.clone()),
            FieldType::MultiSelect { selection_options } => Some(selection_options.clone()),
            _ => None,
        }
    }

    /// returns options as one comma separated string
    /// removing commas from options themselves
    /// avoiding conflicts when posting to halo
    pub fn selection_options_string(&self) -> Option<String> {
        self.selection_options().map(|options| {
            options
                .into_iter()
                .map(|option| option.replace(",", ""))
                .collect::<Vec<String>>()
                .join(", ")
        })
    }

    pub fn field_type_id(&self) -> u8 {
        match self {
            FieldType::Text { .. } => 0,
            FieldType::Memo => 1,
            FieldType::SingleSelect { .. } => 2,
            FieldType::MultiSelect { .. } => 3,
            FieldType::Date { .. } => 4,
            FieldType::Time => 5,
            FieldType::Checkbox => 6,
            FieldType::Rich => 10,
        }
    }
}

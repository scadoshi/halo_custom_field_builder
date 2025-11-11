pub mod field_type;
pub mod label;
pub mod name;

use crate::domain::models::custom_field::{
    field_type::{FieldType, InvalidFieldType},
    label::{InvalidLabel, Label},
    name::{InvalidName, Name},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InvalidCustomField {
    #[error(transparent)]
    Name(InvalidName),
    #[error(transparent)]
    Label(InvalidLabel),
    #[error(transparent)]
    FieldType(InvalidFieldType),
}

impl From<InvalidName> for InvalidCustomField {
    fn from(value: InvalidName) -> Self {
        InvalidCustomField::Name(value)
    }
}

impl From<InvalidLabel> for InvalidCustomField {
    fn from(value: InvalidLabel) -> Self {
        InvalidCustomField::Label(value)
    }
}

impl From<InvalidFieldType> for InvalidCustomField {
    fn from(value: InvalidFieldType) -> Self {
        InvalidCustomField::FieldType(value)
    }
}

#[derive(Debug, Clone)]
pub struct CustomField {
    pub name: Name,
    pub label: Label,
    pub field_type: FieldType,
}
impl CustomField {
    pub fn new(
        name: &str,
        label: &str,
        field_type_id: u8,
        input_type_id: Option<u8>,
        selection_options: Option<String>,
    ) -> Result<Self, InvalidCustomField> {
        let name = Name::new(name)?;
        let label = Label::new(label)?;
        let selection_options = match selection_options {
            Some(options) => options.split(",").map(String::from).collect(),
            None => vec![],
        };
        let field_type = FieldType::new(field_type_id, input_type_id, selection_options)?;

        Ok(Self {
            name,
            label,
            field_type,
        })
    }
}

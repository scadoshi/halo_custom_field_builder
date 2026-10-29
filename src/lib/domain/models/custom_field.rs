//! A custom field built from one CSV row.

pub mod field_type;
pub mod label;
pub mod name;

use thiserror::Error;

use crate::domain::models::custom_field::field_type::{FieldType, ParseFieldTypeError};
use crate::domain::models::custom_field::label::{Label, ParseLabelError};
use crate::domain::models::custom_field::name::{Name, ParseNameError};

/// Why a row is not a valid custom field. Each message starts with the column at fault.
#[derive(Error, Debug)]
pub enum InvalidCustomField {
    /// The `name` column is invalid.
    #[error("name {0}")]
    Name(#[from] ParseNameError),
    /// The `label` column is invalid.
    #[error("label {0}")]
    Label(#[from] ParseLabelError),
    /// The type columns or the options are invalid.
    #[error(transparent)]
    FieldType(#[from] ParseFieldTypeError),
}

/// One field to create, validated.
#[derive(Debug, Clone)]
pub struct CustomField {
    /// The internal name.
    pub name: Name,
    /// What users see.
    pub label: Label,
    /// The type, input type and options.
    pub field_type: FieldType,
}
impl CustomField {
    /// A field from a row's cells. `selection_options` is split on commas.
    ///
    /// # Errors
    ///
    /// Fails when the name, label, type ids or options are invalid, as
    /// [`InvalidCustomField`] names.
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
            Some(options) => options.split(',').map(String::from).collect(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_becomes_a_field_with_its_options_split_on_commas() {
        let field = CustomField::new(
            "pizzaSize",
            "pizza size",
            2,
            Some(0),
            Some("small,medium,large".to_string()),
        )
        .unwrap();
        assert_eq!(field.label.to_string(), "pizza size");
        assert_eq!(
            field.field_type.selection_options(),
            Some(
                &[
                    "small".to_string(),
                    "medium".to_string(),
                    "large".to_string()
                ][..]
            )
        );
    }

    #[test]
    fn each_part_is_checked_and_the_error_says_which() {
        assert!(matches!(
            CustomField::new("pizzaSize", " ", 0, None, None),
            Err(InvalidCustomField::Label(_))
        ));
        assert!(matches!(
            CustomField::new("pizzaSize", "ok", 9, None, None),
            Err(InvalidCustomField::FieldType(_))
        ));
    }

    #[test]
    fn the_error_names_the_column() {
        let message = |name, label, field_type_id, input_type_id| {
            CustomField::new(name, label, field_type_id, input_type_id, None)
                .unwrap_err()
                .to_string()
        };
        assert_eq!(message("", "ok", 0, None), "name cannot be empty");
        assert_eq!(message("ok", " ", 0, None), "label cannot be empty");
        assert_eq!(
            message("ok", "ok", 9, None),
            "field_type_id 9 is not a field type"
        );
        assert_eq!(
            message("ok", "ok", 4, Some(2)),
            "input_type_id 2 is not an input type of field_type_id 4"
        );
    }

    #[test]
    fn options_are_trimmed_and_an_empty_one_is_refused() {
        let field =
            CustomField::new("size", "size", 2, None, Some(" small , large ".to_string())).unwrap();
        assert_eq!(
            field.field_type.selection_options(),
            Some(&["small".to_string(), "large".to_string()][..])
        );
        for options in ["small,,large", "small,", " , "] {
            let message = CustomField::new("size", "size", 3, None, Some(options.to_string()))
                .unwrap_err()
                .to_string();
            assert_eq!(
                message, "selection_options cannot hold an empty option",
                "{options:?}"
            );
        }
    }
}

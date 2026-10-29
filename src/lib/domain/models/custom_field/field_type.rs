pub mod input_types;
use thiserror::Error;

use crate::domain::models::custom_field::field_type::input_types::date_input_type::DateInputType;
use crate::domain::models::custom_field::field_type::input_types::single_select_input_type::SingleSelectInputType;
use crate::domain::models::custom_field::field_type::input_types::text_input_type::TextInputType;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseFieldTypeError {
    #[error("field_type_id {0} is not a field type")]
    InvalidFieldTypeId(u8),
    #[error("input_type_id {input_type_id} is not an input type of field_type_id {field_type_id}")]
    InvalidInputType {
        field_type_id: u8,
        input_type_id: u8,
    },
    #[error("selection_options cannot hold an empty option")]
    EmptyOption,
    #[error("selection_options are only for field_type_id 2 and 3, not {0}")]
    OptionsWithoutSelection(u8),
    #[error("selection_options cannot be empty for field_type_id {0}")]
    SelectionWithoutOptions(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    ) -> Result<Self, ParseFieldTypeError> {
        let selection_options: Vec<String> = selection_options
            .into_iter()
            .map(|option| option.trim().to_string())
            .collect();
        if selection_options.iter().any(String::is_empty) {
            return Err(ParseFieldTypeError::EmptyOption);
        }
        let has_options = !selection_options.is_empty();
        let field_type = match field_type_id {
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
            _ => Err(ParseFieldTypeError::InvalidFieldTypeId(field_type_id)),
        }?;
        match (has_options, field_type.selection_options().is_some()) {
            (true, false) => Err(ParseFieldTypeError::OptionsWithoutSelection(field_type_id)),
            (false, true) => Err(ParseFieldTypeError::SelectionWithoutOptions(field_type_id)),
            _ => Ok(field_type),
        }
    }

    pub fn input_type_id(&self) -> Option<u8> {
        match self {
            FieldType::Text { input_type } => Some(input_type.input_type_id()),
            FieldType::SingleSelect { input_type, .. } => Some(input_type.input_type_id()),
            FieldType::Date { input_type } => Some(input_type.input_type_id()),
            FieldType::Memo
            | FieldType::MultiSelect { .. }
            | FieldType::Time
            | FieldType::Checkbox
            | FieldType::Rich => None,
        }
    }

    pub fn selection_options(&self) -> Option<&[String]> {
        match self {
            FieldType::SingleSelect {
                selection_options, ..
            }
            | FieldType::MultiSelect { selection_options } => Some(selection_options),
            _ => None,
        }
    }

    /// The options joined with `", "`, the form Halo's `new_values` takes.
    /// Halo splits on commas, so a comma inside an option is dropped. Options
    /// read from the CSV never hold one, because the reader splits on commas.
    pub fn selection_options_string(&self) -> Option<String> {
        self.selection_options().map(|options| {
            options
                .iter()
                .map(|option| option.replace(',', ""))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Options for the selection types, which need some, and none for the rest.
    fn options_for(field_type_id: u8) -> Vec<String> {
        match field_type_id {
            2 | 3 => vec!["small".to_string()],
            _ => vec![],
        }
    }

    #[test]
    fn every_field_type_id_round_trips_with_its_input_type_id() {
        let cases: &[(u8, Option<u8>)] = &[
            (0, Some(0)),
            (0, Some(6)),
            (1, None),
            (2, Some(0)),
            (2, Some(2)),
            (3, None),
            (4, Some(0)),
            (4, Some(1)),
            (5, None),
            (6, None),
            (10, None),
        ];
        for (field_type_id, input_type_id) in cases {
            let field_type =
                FieldType::new(*field_type_id, *input_type_id, options_for(*field_type_id))
                    .unwrap();
            assert_eq!(field_type.field_type_id(), *field_type_id);
            assert_eq!(field_type.input_type_id(), *input_type_id);
        }
    }

    #[test]
    fn a_missing_input_type_id_means_the_first_input_type() {
        assert_eq!(
            FieldType::new(0, None, vec![]).unwrap().input_type_id(),
            Some(0)
        );
        assert_eq!(
            FieldType::new(2, None, options_for(2))
                .unwrap()
                .input_type_id(),
            Some(0)
        );
        assert_eq!(
            FieldType::new(4, None, vec![]).unwrap().input_type_id(),
            Some(0)
        );
    }

    #[test]
    fn an_input_type_id_is_ignored_by_types_that_have_none() {
        for id in [1, 3, 5, 6, 10] {
            assert_eq!(
                FieldType::new(id, Some(5), options_for(id))
                    .unwrap()
                    .input_type_id(),
                None
            );
        }
    }

    #[test]
    fn unknown_ids_are_refused() {
        assert_eq!(
            FieldType::new(7, None, vec![]),
            Err(ParseFieldTypeError::InvalidFieldTypeId(7))
        );
        for (field_type_id, input_type_id) in [(0, 7), (2, 3), (4, 2)] {
            assert_eq!(
                FieldType::new(field_type_id, Some(input_type_id), vec![]),
                Err(ParseFieldTypeError::InvalidInputType {
                    field_type_id,
                    input_type_id
                })
            );
        }
    }

    #[test]
    fn selection_options_belong_to_the_selection_types_only() {
        let options = vec!["small".to_string(), "large".to_string()];
        assert_eq!(
            FieldType::new(2, None, options.clone())
                .unwrap()
                .selection_options(),
            Some(options.as_slice())
        );
        assert_eq!(
            FieldType::new(3, None, options.clone())
                .unwrap()
                .selection_options(),
            Some(options.as_slice())
        );
        for id in [0, 1, 4, 5, 6, 10] {
            assert_eq!(
                FieldType::new(id, None, options.clone()),
                Err(ParseFieldTypeError::OptionsWithoutSelection(id))
            );
        }
        for id in [2, 3] {
            assert_eq!(
                FieldType::new(id, None, vec![]),
                Err(ParseFieldTypeError::SelectionWithoutOptions(id))
            );
        }
    }

    #[test]
    fn the_options_string_joins_with_a_comma_and_strips_commas_inside_an_option() {
        let field_type = FieldType::new(3, None, vec!["a,b".to_string(), "c".to_string()]).unwrap();
        assert_eq!(
            field_type.selection_options_string().as_deref(),
            Some("ab, c")
        );
        assert_eq!(FieldType::Memo.selection_options_string(), None);
    }
}

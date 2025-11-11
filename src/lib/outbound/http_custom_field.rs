use crate::domain::models::custom_field::CustomField;
use serde::Serialize;

#[derive(Serialize)]
pub struct HttpCustomField {
    usage: u8,
    name: String,
    label: String,
    #[serde(rename = "type")]
    type_id: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "inputtype")]
    input_type_id: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "new_values")]
    selection_options: Option<String>,
    searchable: bool,
    user_searchable: bool,
    calendar_searchable: bool,
    #[serde(rename = "copytochild")]
    copy_to_child: bool,
    #[serde(rename = "copytochildonupdate")]
    copy_to_child_on_update: bool,
}

impl From<&CustomField> for HttpCustomField {
    fn from(value: &CustomField) -> Self {
        Self {
            usage: 1,
            name: value.name.to_string(),
            label: value.label.to_string(),
            type_id: value.field_type.field_type_id(),
            input_type_id: value.field_type.input_type_id(),
            selection_options: value.field_type.selection_options_string(),
            searchable: true,
            user_searchable: true,
            calendar_searchable: true,
            copy_to_child: true,
            copy_to_child_on_update: true,
        }
    }
}

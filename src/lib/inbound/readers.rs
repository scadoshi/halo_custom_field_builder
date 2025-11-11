use crate::config::Config;
use crate::domain::models::custom_field::CustomField;
use anyhow::{anyhow, Context};
use csv::Reader;

pub struct CsvReader;

#[derive(Debug)]
struct FieldPositions {
    name: usize,
    label: usize,
    field_type_id: usize,
    input_type_id: usize,
    selection_options: usize,
}

impl CsvReader {
    pub fn new() -> Self {
        CsvReader
    }

    fn open_csv(&self, config: &Config) -> anyhow::Result<Reader<std::fs::File>> {
        Ok(Reader::from_path(&config.source_file_name)?)
    }

    fn get_field_positions(&self, headers: &csv::StringRecord) -> anyhow::Result<FieldPositions> {
        Ok(FieldPositions {
            name: headers
                .iter()
                .position(|h| h == "name")
                .ok_or_else(|| anyhow!("missing 'name' column"))?,

            label: headers
                .iter()
                .position(|h| h == "label")
                .ok_or_else(|| anyhow!("missing 'label' column"))?,

            field_type_id: headers
                .iter()
                .position(|h| h == "field_type_id")
                .ok_or_else(|| anyhow!("missing 'field_type_id' column"))?,

            input_type_id: headers
                .iter()
                .position(|h| h == "input_type_id")
                .ok_or_else(|| anyhow!("missing 'input_type_id' column"))?,

            selection_options: headers
                .iter()
                .position(|h| h == "selection_options")
                .ok_or_else(|| anyhow!("missing 'selection_options' column"))?,
        })
    }

    pub fn read_fields(&self, config: &Config) -> anyhow::Result<Vec<CustomField>> {
        let mut fields = Vec::new();
        let mut reader = self.open_csv(config)?;

        let headers = reader.headers()?;
        let positions = self.get_field_positions(headers)?;

        for (raw_row_index, result) in reader.records().enumerate() {
            let row_index = raw_row_index + 2;
            let row_data = result.context(format!("row {}: failed to read entry", row_index))?;

            let field_type_id: u8 = row_data[positions.field_type_id]
                .parse()
                .context(format!("row {}: invalid field_type_id value", row_index))?;

            let input_type_id: Option<u8> = if row_data[positions.input_type_id].trim().is_empty() {
                None
            } else {
                Some(
                    row_data[positions.input_type_id]
                        .parse()
                        .context(format!("row {}: invalid input_type_id value", row_index))?,
                )
            };

            let selection_options = if row_data[positions.selection_options].trim().is_empty() {
                None
            } else {
                Some(row_data[positions.selection_options].to_string())
            };

            let field = CustomField::new(
                &row_data[positions.name],
                &row_data[positions.label],
                field_type_id,
                input_type_id,
                selection_options,
            )
            .context(format!(
                "row {}: failed to create custom field",
                row_index + 2
            ))?;

            fields.push(field);
        }

        Ok(fields)
    }
}

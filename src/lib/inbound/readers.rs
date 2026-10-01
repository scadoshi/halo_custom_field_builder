use crate::config::Config;
use crate::domain::models::custom_field::CustomField;
use anyhow::{Context, anyhow};
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

impl Default for CsvReader {
    fn default() -> Self {
        Self::new()
    }
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
            .context(format!("row {}: failed to create custom field", row_index))?;

            fields.push(field);
        }

        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(csv: &str) -> anyhow::Result<Vec<CustomField>> {
        let path = std::env::temp_dir().join(format!(
            "halo_fields_{}_{}.csv",
            std::process::id(),
            csv.len()
        ));
        std::fs::write(&path, csv).unwrap();
        let config = Config::from_lookup(|key| match key {
            "BASE_URL" => Some("https://example.halo.com".to_string()),
            "SOURCE_FILE_NAME" => Some(path.to_string_lossy().into_owned()),
            _ => Some("x".to_string()),
        })
        .unwrap();
        let result = CsvReader::new().read_fields(&config);
        std::fs::remove_file(&path).unwrap();
        result
    }

    #[test]
    fn columns_are_found_by_header_in_any_order() {
        let fields = read(
            "label,selection_options,name,input_type_id,field_type_id\n\
             pizza size,\"small,large\",pizzaSize,0,2\n\
             notes,,notes,,1\n",
        )
        .unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].label.to_string(), "pizza size");
        assert_eq!(fields[0].field_type.field_type_id(), 2);
        assert_eq!(
            fields[0].field_type.selection_options(),
            Some(vec!["small".to_string(), "large".to_string()])
        );
        assert_eq!(fields[1].field_type.field_type_id(), 1);
        assert_eq!(fields[1].field_type.input_type_id(), None);
    }

    #[test]
    fn a_missing_column_is_named() {
        let message = read("name,label,field_type_id,input_type_id\na,b,0,0\n")
            .unwrap_err()
            .to_string();
        assert!(message.contains("selection_options"), "{message}");
    }

    #[test]
    fn a_bad_row_is_reported_by_its_line_in_the_file() {
        let csv = "name,label,field_type_id,input_type_id,selection_options\n\
                   fine,fine,0,0,\n\
                   broken,broken,nine,0,\n";
        assert!(read(csv).unwrap_err().to_string().contains("row 3"));

        let csv = "name,label,field_type_id,input_type_id,selection_options\n\
                   fine,fine,0,0,\n\
                   fine,fine,0,0,\n\
                   broken,broken,9,0,\n";
        let message = format!("{:#}", read(csv).unwrap_err());
        assert!(message.contains("row 4"), "{message}");
    }
}

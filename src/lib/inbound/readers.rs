use std::fs::File;
use std::path::Path;

use anyhow::{Context, anyhow};
use csv::{Position, Reader, StringRecord};

use crate::domain::models::custom_field::CustomField;

#[derive(Debug)]
struct FieldPositions {
    name: usize,
    label: usize,
    field_type_id: usize,
    input_type_id: usize,
    selection_options: usize,
}

fn open_csv(path: &Path) -> anyhow::Result<Reader<File>> {
    Reader::from_path(path).with_context(|| format!("failed to open {}", path.display()))
}

fn column_positions(headers: &StringRecord) -> anyhow::Result<FieldPositions> {
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

pub fn read_fields(path: &Path) -> anyhow::Result<Vec<CustomField>> {
    let mut fields = Vec::new();
    let mut reader = open_csv(path)?;

    let headers = reader.headers()?;
    let positions = column_positions(headers)?;

    for (index, result) in reader.records().enumerate() {
        // The line a record starts on; counting records misses multi-line cells.
        let fallback = u64::try_from(index).unwrap_or(u64::MAX).saturating_add(2);
        let row_index = match &result {
            Ok(row) => row.position(),
            Err(e) => e.position(),
        }
        .map_or(fallback, Position::line);
        let row_data = result.context(format!("row {}: failed to read entry", row_index))?;
        let cell = |index| column(&row_data, index).with_context(|| format!("row {row_index}"));

        let field_type_id: u8 = cell(positions.field_type_id)?
            .parse()
            .context(format!("row {}: invalid field_type_id value", row_index))?;

        let input_type_id = cell(positions.input_type_id)?;
        let input_type_id: Option<u8> = if input_type_id.trim().is_empty() {
            None
        } else {
            Some(
                input_type_id
                    .parse()
                    .context(format!("row {}: invalid input_type_id value", row_index))?,
            )
        };

        let selection_options = cell(positions.selection_options)?;
        let selection_options = if selection_options.trim().is_empty() {
            None
        } else {
            Some(selection_options.to_string())
        };

        let field = CustomField::new(
            cell(positions.name)?,
            cell(positions.label)?,
            field_type_id,
            input_type_id,
            selection_options,
        )
        .context(format!("row {}: failed to create custom field", row_index))?;

        fields.push(field);
    }

    Ok(fields)
}

/// The cell at `index`, or an error when the row is too short to have one.
fn column(row: &StringRecord, index: usize) -> anyhow::Result<&str> {
    row.get(index).with_context(|| {
        format!(
            "no column {} in a row of {}",
            index.saturating_add(1),
            row.len()
        )
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn read(csv: &str) -> anyhow::Result<Vec<CustomField>> {
        // Tests run in parallel, so each call writes its own file.
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "halo_fields_{}_{}.csv",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, csv).unwrap();
        let result = read_fields(&path);
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

    #[test]
    fn a_missing_file_is_named() {
        let path = std::env::temp_dir().join("halo_fields_that_do_not_exist.csv");
        let message = format!("{:#}", read_fields(&path).unwrap_err());
        assert!(message.contains(&*path.to_string_lossy()), "{message}");
    }

    #[test]
    fn a_short_row_is_an_error_not_a_panic() {
        let row = StringRecord::from(vec!["a", "b"]);
        assert_eq!(column(&row, 1).unwrap(), "b");
        assert_eq!(
            column(&row, 2).unwrap_err().to_string(),
            "no column 3 in a row of 2"
        );
    }

    #[test]
    fn options_on_a_type_without_options_are_refused_with_the_row() {
        let csv = "name,label,field_type_id,input_type_id,selection_options\n\
                   size,size,2,0,\"small,large\"\n\
                   notes,notes,1,,\"small,large\"\n";
        let message = format!("{:#}", read(csv).unwrap_err());
        assert!(message.contains("row 3"), "{message}");
        assert!(message.contains("selection_options"), "{message}");
    }

    #[test]
    fn a_selection_type_without_options_is_refused_with_the_row() {
        for field_type_id in [2, 3] {
            let csv = format!(
                "name,label,field_type_id,input_type_id,selection_options\n\
                 size,size,2,0,\"small,large\"\n\
                 empty,empty,{field_type_id},,\n"
            );
            let message = format!("{:#}", read(&csv).unwrap_err());
            assert!(message.starts_with("row 3:"), "{message}");
            assert!(message.contains("selection_options"), "{message}");
        }
    }

    #[test]
    fn a_row_after_a_multi_line_cell_is_reported_by_its_line_in_the_file() {
        let csv = "name,label,field_type_id,input_type_id,selection_options\n\
                   notes,\"two\nlines\",1,,\n\
                   broken,broken,nine,0,\n";
        let message = format!("{:#}", read(csv).unwrap_err());
        assert!(message.starts_with("row 4:"), "{message}");
    }

    #[test]
    fn a_row_with_the_wrong_number_of_cells_is_reported_by_its_line() {
        let csv = "name,label,field_type_id,input_type_id,selection_options\n\
                   notes,\"two\nlines\",1,,\n\
                   short,short\n";
        let message = format!("{:#}", read(csv).unwrap_err());
        assert!(message.starts_with("row 4:"), "{message}");
    }
}

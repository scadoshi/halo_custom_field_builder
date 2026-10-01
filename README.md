# halo custom field builder

creates custom fields in Halo from a CSV, one API call per row. built in Rust, shipped as a single Windows executable.

## setup

a `.env` next to the executable, no quotes around values:

```env
BASE_URL=https://your-instance.halo.com
CLIENT_ID=dd5ef51d-ec0f-4247-b79d-1234b0e40dec
CLIENT_SECRET=8595ec7e-81e5-4a17-1234-6c3ae166e0c7
SOURCE_FILE_NAME=source.csv
```

the token and API URLs are built from `BASE_URL`. the CSV named by `SOURCE_FILE_NAME` sits next to the executable too. a missing variable stops the program and names it.

## the CSV

exactly these columns, in this order:

```
name,label,field_type_id,input_type_id,selection_options
```

- `name`: letters, digits and underscores, up to 64 characters
- `label`: what users see; not empty
- `field_type_id` and `input_type_id`: from the tables below
- `selection_options`: comma-separated choices for field types 2 and 3, empty otherwise

the shipped `source.csv` is a complete example, a pizza order form with every field type in it.

### field types

| field type         | field_type_id | has input types |
| ------------------ | ------------- | --------------- |
| text               | 0             | yes             |
| memo               | 1             | no              |
| single selection   | 2             | yes             |
| multiple selection | 3             | no              |
| date               | 4             | yes             |
| time               | 5             | no              |
| checkbox           | 6             | no              |
| rich               | 10            | no              |

a type with no input types takes `input_type_id` 0.

### input types

text (field_type_id 0):

| input type   | input_type_id |
| ------------ | ------------- |
| anything     | 0             |
| integer      | 1             |
| money        | 2             |
| alphanumeric | 3             |
| decimal      | 4             |
| URL          | 5             |
| password     | 6             |

single selection (field_type_id 2):

| input type        | input_type_id |
| ----------------- | ------------- |
| standard dropdown | 0             |
| tree dropdown     | 1             |
| radio selection   | 2             |

date (field_type_id 4):

| input type | input_type_id |
| ---------- | ------------- |
| date       | 0             |
| datetime   | 1             |

## running

open a command prompt in the program's directory and run `halo_custom_field_builder.exe`. a menu offers three choices:

1. import all fields
2. debug mode: one field at a time, shown before it is sent, with skip and quit
3. quit

a `.bat` containing `cmd /k halo_custom_field_builder.exe` makes a double-click launcher. it is not in the distribution because antivirus software tends to flag batch files; make it yourself.

## pacing

Halo allows 700 requests per rolling five minutes. the program waits 500 ms between field creations, which comes to about one field a second once the API's own time is counted: 100 fields in two minutes, 1,000 in about seventeen.

## errors and logs

a CSV problem is reported with its row number before anything is sent. an API failure is reported per field and the run continues. everything is also written to `logs/`, one file per run; files older than seven days are deleted, and never more than 100 are kept.

## limits

- creates fields only; it does not update or delete
- every field gets Halo's default usage and searchable settings

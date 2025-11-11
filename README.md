# halo custom field builder

CLI tool for bulk creation of custom fields in Halo using CSV input. built with Rust.

## features

- environment configuration validation with clear error messages
- CSV input validation against Halo field requirements  
- type-safe domain models with compile-time guarantees
- OAuth 2.0 authentication with automatic token refresh
- rate-limited API requests (500ms between calls)
- interactive debug mode for field-by-field review
- automatic log rotation (7 days retention, max 100 files)
- detailed error context for troubleshooting

## requirements

- `.env` configuration file
- CSV input file with proper column headers
- no additional runtime dependencies (standalone executable)

## configuration

### environment variables

create a `.env` file in the same directory as the executable:

| variable           | required | description                     |
| ------------------ | -------- | ------------------------------- |
| `BASE_URL`         | yes      | Halo instance URL (HTTPS only) |
| `CLIENT_ID`        | yes      | OAuth 2.0 client identifier     |
| `CLIENT_SECRET`    | yes      | OAuth 2.0 client secret         |
| `SOURCE_FILE_NAME` | yes      | CSV input filename              |

### example configuration

```env
BASE_URL=https://your-instance.halo.com
CLIENT_ID=dd5ef51d-ec0f-4247-b79d-1234b0e40dec
CLIENT_SECRET=8595ec7e-81e5-4a17-1234-6c3ae166e0c7
SOURCE_FILE_NAME=source.csv
```

**notes:**
- do not use quotes around values
- BASE_URL must use HTTPS
- API and auth URLs are automatically generated from BASE_URL
- file must be in same directory as executable

## CSV format

### required columns

CSV must contain exactly these columns:

```
name,label,field_type_id,input_type_id,selection_options
```

### column specifications

**name**
- alphanumeric characters and underscores only
- no spaces or special characters
- maximum 64 characters

**label**  
- any visible characters allowed
- cannot be empty or single space

**field_type_id**
- valid values: 0, 1, 2, 3, 4, 5, 6, 10
- see field types table below

**input_type_id**
- depends on field_type_id (see field types table)
- can be empty for types with no input options

**selection_options**
- required for field_type_id 2 and 3 (selection fields)
- comma-separated list of choices
- empty for other field types

### field type reference

#### basic field types

| field type         | field_type_id | has input types   |
| ------------------ | ------------- | ----------------- |
| text               | 0             | yes               |
| memo               | 1             | no                |
| single selection   | 2             | yes               |
| multiple selection | 3             | no                |
| date               | 4             | yes               |
| time               | 5             | no                |
| checkbox           | 6             | no                |
| rich               | 10            | no                |

#### input options by field type

**text field input types** (field_type_id: 0)

| input type   | input_type_id | description                 |
| ------------ | ------------- | --------------------------- |
| anything     | 0             | any text input              |
| integer      | 1             | numbers only                |
| money        | 2             | currency format             |
| alphanumeric | 3             | letters and numbers only    |
| decimal      | 4             | numbers with decimal places |
| URL          | 5             | web address format          |
| password     | 6             | masked input field          |

**single selection input types** (field_type_id: 2)

| input type        | input_type_id | description           |
| ----------------- | ------------- | --------------------- |
| standard dropdown | 0             | basic dropdown menu   |
| tree dropdown     | 1             | hierarchical dropdown |
| radio selection   | 2             | radio button options  |

**date field input types** (field_type_id: 4)

| input type | input_type_id | description   |
| ---------- | ------------- | ------------- |
| date       | 0             | date only     |
| datetime   | 1             | date and time |

**fields with no input options** (always use input_type_id: 0)

- memo (field_type_id: 1)
- multiple selection (field_type_id: 3)
- time (field_type_id: 5)
- checkbox (field_type_id: 6)
- rich (field_type_id: 10)

### example fields

sample configuration for a pizza ordering system:

| name                | label                | field_type_id | input_type_id | selection_options                                                                                                                                                                                                           |
| ------------------- | -------------------- | ------------- | ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| orderName           | order name           | 0             | 0             |                                                                                                                                                                                                                             |
| orderPhone          | phone number         | 0             | 1             |                                                                                                                                                                                                                             |
| pizzaSize           | pizza size           | 2             | 0             | small,medium,large                                                                                                                                                                                                          |
| crustType           | crust type           | 2             | 0             | thin,regular,deep dish,stuffed                                                                                                                                                                                              |
| toppings            | toppings             | 3             | 0             | pepperoni,mushrooms,pineapple,sausage,green peppers,red onions,black olives,bacon,ham,ground beef,italian sausage,spinach,fresh tomatoes,jalapeños,anchovies,chicken,feta,extra mozzarella,roasted garlic,artichoke hearts |
| extraCheese         | extra cheese         | 6             | 0             |                                                                                                                                                                                                                             |
| specialInstructions | special instructions | 1             | 0             |                                                                                                                                                                                                                             |
| allergyNotes        | allergy information  | 10            | 0             |                                                                                                                                                                                                                             |
| deliveryDate        | delivery date        | 4             | 0             |                                                                                                                                                                                                                             |
| deliveryTime        | preferred time       | 5             | 0             |                                                                                                                                                                                                                             |
| paymentType         | payment type         | 2             | 2             | cash,card,check                                                                                                                                                                                                             |
| orderTip            | tip                  | 0             | 4             |                                                                                                                                                                                                                             |

## rate limiting

### API constraints

the Halo API implements rate limiting of 700 requests per 5-minute rolling window. to ensure reliable operation and prevent throttling, this program implements a conservative rate limiting strategy:

- enforces 500ms delay between each field creation request
- results in approximately 120 requests per minute
- stays well under the API limit of 700 requests per 5 minutes
- no manual throttling required

### processing time estimates

due to rate limiting and API processing time:

- each field takes approximately 1 second to process (500ms enforced delay + API response time)
- 100 fields ≈ 2 minutes
- 500 fields ≈ 10 minutes
- 1000 fields ≈ 17 minutes (based on actual testing)

real-world testing with 1000 fields completed in approximately 17 minutes, accounting for:

- 500ms enforced delay between requests
- Halo API processing time
- network latency
- response handling

this controlled pacing helps ensure:

- reliable field creation
- no API throttling errors
- predictable processing times
- minimal impact on API performance

## error handling

the program includes comprehensive error handling for:

- environment configuration issues
- CSV file validation
- API authentication
- field creation failures

each error provides specific details about:

- location of the error (row number for CSV errors)
- nature of the problem
- suggested fixes where applicable

## logging

the program maintains detailed logs of all operations:

- logs stored in the `logs` directory
- automatic rotation (7 days retention)
- maximum of 100 log files retained
- each log includes:
  - timestamp
  - operation type
  - success/failure status
  - detailed error messages when applicable

## debug mode

the program includes a debug mode that allows you to:

- process fields one at a time
- review field details before processing
- skip specific fields
- get immediate feedback on success/failure
- exit at any point

## distribution

the program distribution includes:

| file/folder                     | purpose                 | notes                                 |
| ------------------------------- | ----------------------- | ------------------------------------- |
| `halo_custom_field_builder.exe` | main executable         | core program                          |
| `source.csv`                    | sample input CSV file   | example with all field types          |
| `README.md`                     | documentation           | contains setup and usage instructions |
| `logs/`                         | directory for log files | created automatically on first run    |

**note:** you will need to create your own `.env` file (see configuration section above)

### file locations

| requirement          | description                                             |
| -------------------- | ------------------------------------------------------- |
| `.env` location      | must be in the same directory as the executable         |
| source file location | must be in the same directory as the executable         |
| logs directory       | created automatically on first run in program directory |

## running the program

### direct execution

1. open command prompt in program directory (Windows + R, type "cmd", Enter)
2. run: `halo_custom_field_builder.exe`

### optional batch file setup

1. create a new `.bat` file containing:
   ```batch
   cmd /k halo_custom_field_builder.exe
   ```
2. save as `run_halo_custom_field_builder.bat` in program directory
3. double-click to run

**note:** the `.bat` file is not included in the distributable since antivirus software often flags batch files. you can safely create this launcher yourself following the steps above, or simply use the direct execution method. the `.bat` file enables running via shortcuts from any location.

## limitations

- program currently only supports field creation (not updating or deleting)
- all fields created with default usage and searchable settings
- batch processing limited to one field at a time to ensure proper error handling

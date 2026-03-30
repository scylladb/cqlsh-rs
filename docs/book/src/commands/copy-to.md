# COPY TO

Export table data to a CSV file or stdout.

## Syntax

```
COPY [keyspace.]table [(column1, column2, ...)] TO 'filename'|STDOUT [WITH option = value [AND ...]]
```

## Options

| Option | Default | Description |
|--------|---------|-------------|
| `DELIMITER` | `,` | Field delimiter character |
| `QUOTE` | `"` | Quote character for fields |
| `ESCAPE` | `\\` | Escape character |
| `HEADER` | `false` | Include column headers in first row |
| `NULL` | (empty) | String to represent NULL values |
| `DATETIMEFORMAT` | from config | Timestamp format |
| `ENCODING` | `utf-8` | Output encoding |
| `FLOATPRECISION` | `5` | Decimal digits for float values |
| `DOUBLEPRECISION` | `12` | Decimal digits for double values |
| `DECIMALSEP` | `.` | Decimal separator |
| `THOUSANDSSEP` | | Thousands separator |
| `BOOLSTYLE` | `True,False` | Boolean representation |
| `PAGESIZE` | `1000` | Rows per page |
| `MAXOUTPUTSIZE` | unlimited | Maximum rows to export |

## Examples

```sql
-- Export entire table
COPY users TO '/tmp/users.csv';

-- Export specific columns with headers
COPY users (id, name, email) TO '/tmp/users.csv' WITH HEADER = true;

-- Export to stdout
COPY users TO STDOUT;

-- Export with custom delimiter
COPY users TO '/tmp/users.tsv' WITH DELIMITER = '\t' AND HEADER = true;
```

## Notes

- Disabled when `--no-file-io` is used (except `TO STDOUT`).
- See the [COPY Guide](../copy-guide.md) for detailed usage and performance tips.

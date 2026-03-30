# COPY FROM

Import CSV data into a table.

## Syntax

```
COPY [keyspace.]table [(column1, column2, ...)] FROM 'filename'|STDIN [WITH option = value [AND ...]]
```

## Options

| Option | Default | Description |
|--------|---------|-------------|
| `DELIMITER` | `,` | Field delimiter character |
| `QUOTE` | `"` | Quote character |
| `ESCAPE` | `\\` | Escape character |
| `HEADER` | `false` | First row contains column headers |
| `NULL` | (empty) | String representing NULL values |
| `CHUNKSIZE` | `1000` | Rows per insert batch |
| `INGESTRATE` | `100000` | Target rows per second |
| `MAXBATCHSIZE` | `20` | Maximum rows per batch statement |
| `MINBATCHSIZE` | `2` | Minimum rows per batch statement |
| `MAXPARSEERRORS` | `-1` | Max parse errors before abort (-1 = unlimited) |
| `MAXINSERTERRORS` | `-1` | Max insert errors before abort (-1 = unlimited) |
| `PREPAREDSTATEMENTS` | `true` | Use prepared statements |
| `TTL` | `-1` | TTL for inserted rows in seconds (-1 = no TTL) |

## Examples

```sql
-- Import from file
COPY users FROM '/tmp/users.csv';

-- Import with headers
COPY users FROM '/tmp/users.csv' WITH HEADER = true;

-- Import specific columns
COPY users (id, name) FROM '/tmp/users.csv' WITH HEADER = true;

-- Import from stdin
COPY users FROM STDIN;
```

## Notes

- Disabled when `--no-file-io` is used (except `FROM STDIN`).
- See the [COPY Guide](../copy-guide.md) for detailed usage and performance tips.

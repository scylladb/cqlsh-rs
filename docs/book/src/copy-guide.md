# COPY Guide

Comprehensive guide to importing and exporting data with COPY TO and COPY FROM.

## Exporting data with COPY TO

### Basic export

```sql
COPY my_keyspace.users TO '/tmp/users.csv';
```

### Export with headers

```sql
COPY users TO '/tmp/users.csv' WITH HEADER = true;
```

### Export specific columns

```sql
COPY users (id, name, email) TO '/tmp/users.csv' WITH HEADER = true;
```

### Export to stdout

Useful for piping to other tools:

```sql
COPY users TO STDOUT WITH HEADER = true;
```

### Tab-separated values

```sql
COPY users TO '/tmp/users.tsv' WITH DELIMITER = '\t';
```

### Custom NULL representation

```sql
COPY users TO '/tmp/users.csv' WITH NULL = 'N/A';
```

## Importing data with COPY FROM

### Basic import

```sql
COPY my_keyspace.users FROM '/tmp/users.csv';
```

### Import with headers

If the CSV has a header row:

```sql
COPY users FROM '/tmp/users.csv' WITH HEADER = true;
```

### Import specific columns

Map CSV columns to table columns:

```sql
COPY users (id, name) FROM '/tmp/users.csv' WITH HEADER = true;
```

### Import from stdin

```sql
COPY users FROM STDIN;
```

### Set TTL on imported rows

```sql
COPY users FROM '/tmp/users.csv' WITH TTL = 86400;
```

## Performance tuning

### COPY TO performance

- **PAGESIZE**: Increase for faster exports on large tables. Default: 1000.
- **MAXOUTPUTSIZE**: Limit the number of exported rows for sampling.

```sql
COPY users TO '/tmp/users.csv' WITH PAGESIZE = 5000;
```

### COPY FROM performance

- **CHUNKSIZE**: Number of rows per batch. Increase for faster imports, decrease if hitting timeouts.
- **INGESTRATE**: Target rows per second. Reduce if the cluster is under load.
- **MAXBATCHSIZE**: Maximum rows per batch statement. Smaller batches are safer.
- **PREPAREDSTATEMENTS**: Keep `true` for better performance.

```sql
COPY users FROM '/tmp/users.csv' WITH CHUNKSIZE = 5000 AND INGESTRATE = 50000;
```

### Error handling

- **MAXPARSEERRORS**: Maximum CSV parse errors before aborting (-1 = unlimited).
- **MAXINSERTERRORS**: Maximum insert errors before aborting (-1 = unlimited).

```sql
COPY users FROM '/tmp/users.csv' WITH MAXPARSEERRORS = 100 AND MAXINSERTERRORS = 50;
```

## Configuration defaults

Default COPY options can be set in the cqlshrc file. See [Configuration: copy](./configuration/copy.md).

## Common issues

### Large text/blob fields

If you get errors with large fields, increase the CSV field size limit:

```ini
[csv]
field_size_limit = 1048576
```

### Timeout errors

Reduce batch size and ingest rate:

```sql
COPY users FROM '/tmp/users.csv' WITH CHUNKSIZE = 100 AND INGESTRATE = 10000;
```

### Encoding issues

Specify the encoding explicitly:

```sql
COPY users TO '/tmp/users.csv' WITH ENCODING = 'utf-8';
```

# Configuration Reference

cqlsh-rs reads configuration from `~/.cassandra/cqlshrc` by default. Override the path with `--cqlshrc`.

This is the same INI-format configuration file used by the Python `cqlsh`. All sections and keys are fully compatible.

## File format

The cqlshrc file uses standard INI format:

```ini
[section]
key = value
```

## Example cqlshrc

```ini
[authentication]
username = cassandra
password = cassandra

[connection]
hostname = 127.0.0.1
port = 9042
connect_timeout = 5
request_timeout = 10

[ssl]
certfile = /path/to/ca-cert.pem
validate = true

[ui]
color = on
datetimeformat = %Y-%m-%d %H:%M:%S%z
float_precision = 5
encoding = utf-8

[csv]
field_size_limit = 131072

[copy]
numprocesses = 4
maxattempts = 5

[copy-to]
pagesize = 1000
pagetimeout = 10

[copy-from]
chunksize = 1000
ingestrate = 100000

[tracing]
max_trace_wait = 10.0
```

## Sections

| Section | Description |
|---------|-------------|
| [`[authentication]`](authentication.md) | Username, password, credentials file |
| [`[connection]`](connection.md) | Host, port, timeouts |
| [`[ssl]`](ssl.md) | SSL/TLS certificates and validation |
| [`[ui]`](ui.md) | Display settings (color, formats, encoding) |
| [`[cql]`](cql.md) | CQL version |
| [`[csv]`](csv.md) | CSV field size limits |
| [`[copy]` / `[copy-to]` / `[copy-from]`](copy.md) | COPY command defaults |
| [`[tracing]`](tracing.md) | Tracing wait times |

## Precedence

Configuration values are resolved in this order (highest priority first):

1. CLI flags
2. Environment variables
3. cqlshrc file values
4. Built-in defaults

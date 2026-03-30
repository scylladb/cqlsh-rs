# SHOW

Display version, host, or session trace information.

## Syntax

```
SHOW VERSION
SHOW HOST
SHOW SESSION <trace-uuid>
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `VERSION` | Show the cqlsh-rs version and connected Cassandra version |
| `HOST` | Show the connected host and port |
| `SESSION <uuid>` | Display the trace for a previously traced request |

## Usage

```
cqlsh> SHOW VERSION
cqlsh-rs 0.1.0 | Cassandra 4.1.0 | CQL spec 3.4.6

cqlsh> SHOW HOST
Connected to Test Cluster at 127.0.0.1:9042

cqlsh> SHOW SESSION 12345678-1234-1234-1234-123456789abc
-- trace output --
```

## Notes

- `SHOW SESSION` requires a valid trace UUID from a previously traced request (enable with `TRACING ON`).

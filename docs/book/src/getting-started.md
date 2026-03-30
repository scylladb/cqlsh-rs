# Getting Started

This guide walks you through installing cqlsh-rs and connecting to your first Cassandra cluster.

## Prerequisites

- A running Apache Cassandra, ScyllaDB, or compatible database
- Network access to the database's native transport port (default: 9042)

## Quick install

The fastest way to get started:

```bash
cargo install --git https://github.com/fruch/cqlsh-rs.git
```

See [Installation](./installation.md) for more options (Homebrew, Docker, pre-built binaries).

## Connect to a cluster

```bash
# Connect to localhost:9042
cqlsh-rs

# Connect to a specific host
cqlsh-rs 10.0.0.1

# Connect to a specific host and port
cqlsh-rs 10.0.0.1 9043
```

## Run your first query

Once connected, you'll see the interactive prompt:

```
cqlsh>
```

Try a few commands:

```sql
-- Show cluster info
DESCRIBE CLUSTER;

-- List keyspaces
DESCRIBE KEYSPACES;

-- Query system tables
SELECT cluster_name, release_version FROM system.local;
```

## Execute a single statement

Use `-e` to run a statement and exit:

```bash
cqlsh-rs -e "SELECT * FROM system.local"
```

## Execute a CQL file

Use `-f` to run statements from a file:

```bash
cqlsh-rs -f schema.cql
```

## Connect with authentication

```bash
cqlsh-rs -u cassandra -p cassandra
```

## Connect with SSL/TLS

```bash
cqlsh-rs --ssl
```

See [Configuration Reference](./configuration/ssl.md) for full SSL setup.

## Use a specific keyspace

```bash
cqlsh-rs -k my_keyspace
```

Or switch keyspaces inside the shell:

```sql
USE my_keyspace;
```

## Next steps

- [CLI Reference](./cli-reference.md) — all command-line flags
- [Command Reference](./commands/index.md) — shell commands (DESCRIBE, COPY, etc.)
- [Configuration Reference](./configuration/index.md) — cqlshrc file format
- [Migration Guide](./migration.md) — moving from Python cqlsh

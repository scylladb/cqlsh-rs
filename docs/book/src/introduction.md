# cqlsh-rs

**cqlsh-rs** is a ground-up Rust re-implementation of the Python `cqlsh` — the official interactive CQL shell for [Apache Cassandra](https://cassandra.apache.org/) and compatible databases (ScyllaDB, Amazon Keyspaces, Astra DB).

## Why cqlsh-rs?

- **Single static binary** — no Python runtime, no pip, no virtualenv
- **Fast startup** — launches in milliseconds, not seconds
- **100% CLI compatible** — drop-in replacement for the Python `cqlsh`
- **Modern terminal experience** — syntax highlighting, tab completion, pager integration
- **Cross-platform** — Linux, macOS, Windows

## How to use this documentation

- **New users**: Start with [Getting Started](./getting-started.md)
- **Installing**: See [Installation](./installation.md) for all platforms
- **Migrating from Python cqlsh**: Read the [Migration Guide](./migration.md)
- **Reference**: Browse [CLI Reference](./cli-reference.md), [Command Reference](./commands/README.md), and [Configuration Reference](./configuration/README.md)
- **Having issues?**: Check [Troubleshooting](./troubleshooting.md)

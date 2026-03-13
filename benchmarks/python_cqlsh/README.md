# Python cqlsh Benchmarks

Comparative benchmarks measuring the Python `cqlsh` (from PyPI) against a live
ScyllaDB instance. These serve as the baseline for comparing against the Rust
`cqlsh-rs` implementation.

## Prerequisites

- Docker (for ScyllaDB testcontainers)
- Python 3.10+
- [uv](https://docs.astral.sh/uv/)

## Quick Start

```bash
cd benchmarks/python_cqlsh

# Install dependencies
uv sync

# Run benchmarks (starts ScyllaDB container automatically)
uv run pytest benchmarks/ --benchmark-enable -v
```

## Benchmark Groups

| Group                | What it Measures                                |
|----------------------|-------------------------------------------------|
| `version`            | `cqlsh --version` — pure startup cost           |
| `connect-query`      | Connect + `SELECT now() FROM system.local`      |
| `select-single`      | SELECT one row by primary key                   |
| `select-multi`       | SELECT 100 rows                                 |
| `insert`             | INSERT a single row                             |
| `describe`           | DESCRIBE KEYSPACES                              |
| `select-system-schema` | SELECT from system_schema.tables              |

## Saving Results

```bash
# Save benchmark results as JSON
uv run pytest benchmarks/ --benchmark-enable --benchmark-json=results.json

# Compare against a previous run
uv run pytest benchmarks/ --benchmark-enable --benchmark-compare=results.json
```

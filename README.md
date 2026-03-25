# cqlsh-rs

A ground-up Rust re-implementation of the Python `cqlsh` — the official interactive CQL shell for [Apache Cassandra](https://cassandra.apache.org/) and compatible databases (ScyllaDB, Amazon Keyspaces, Astra DB).

The goal is **100% command-line and configuration compatibility** with the original Python cqlsh, delivered as a single static binary with zero runtime dependencies.

> **Status:** Early development (Phase 1 — Bootstrap MVP in progress).

## Development Progress

<p align="center">
  <img src="docs/assets/progress-roadmap.svg" alt="cqlsh-rs Development Roadmap" width="880"/>
</p>

<details>
<summary>How to update progress</summary>

Edit `docs/progress.json` with updated task counts and velocity data. The roadmap SVG is auto-regenerated when PRs that modify `docs/progress.json` are merged into main.

You can also regenerate locally:
```bash
python3 scripts/generate_progress_svg.py
```
</details>

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70+ (2021 edition)
- `cargo` (included with Rust)

## Building from source

```bash
git clone https://github.com/fruch/cqlsh-rs.git
cd cqlsh-rs
cargo build --release
```

The binary is at `target/release/cqlsh-rs`.

## Installation

### From source (via cargo)

```bash
cargo install --path .
```

This installs `cqlsh-rs` into `~/.cargo/bin/`.

## Usage

```bash
# Connect to localhost:9042 (default)
cqlsh-rs

# Connect to a specific host and port
cqlsh-rs 10.0.0.1 9043

# Execute a single statement and exit
cqlsh-rs -e "SELECT * FROM system.local"

# Execute statements from a file
cqlsh-rs -f schema.cql

# Connect with authentication
cqlsh-rs -u cassandra -p cassandra

# Connect with SSL/TLS
cqlsh-rs --ssl

# Use a specific keyspace
cqlsh-rs -k my_keyspace

# Use a custom cqlshrc configuration file
cqlsh-rs --cqlshrc /path/to/cqlshrc

# Set timeouts
cqlsh-rs --connect-timeout 30 --request-timeout 60
```

### Environment variables

| Variable | Description |
|----------|-------------|
| `CQLSH_HOST` | Default contact point hostname |
| `CQLSH_PORT` | Default native transport port |
| `SSL_CERTFILE` | SSL certificate file path |
| `SSL_VALIDATE` | Enable/disable certificate validation |
| `CQLSH_DEFAULT_CONNECT_TIMEOUT_SECONDS` | Default connect timeout |
| `CQLSH_DEFAULT_REQUEST_TIMEOUT_SECONDS` | Default request timeout |
| `CQL_HISTORY` | Override history file path |

### Configuration file

cqlsh-rs reads `~/.cassandra/cqlshrc` by default (override with `--cqlshrc`). This is the same INI-format configuration file used by the Python cqlsh:

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
```

Configuration precedence: **CLI flags > environment variables > cqlshrc > defaults**.

### Shell completions

Generate shell completion scripts for your shell:

```bash
# Bash
cqlsh-rs --completions bash > /etc/bash_completion.d/cqlsh-rs

# Zsh
cqlsh-rs --completions zsh > ~/.zfunc/_cqlsh-rs

# Fish
cqlsh-rs --completions fish > ~/.config/fish/completions/cqlsh-rs.fish
```

## Benchmarks

Performance is tracked continuously via CI. Results are available at:

- **[Historical Dashboard](https://fruch.github.io/cqlsh-rs/dev/bench/)** — Interactive commit-over-commit charts (updated on every merge to main)
- **[Benchmark Workflow Runs](https://github.com/fruch/cqlsh-rs/actions/workflows/bench.yml)** — Grouped benchmark tables and Criterion artifacts posted to each CI run's summary page
- **[Criterion Reports](https://github.com/fruch/cqlsh-rs/actions/workflows/bench.yml)** — Detailed HTML reports uploaded as artifacts on each run (retained 90 days)
- **Rust vs Python** — Hyperfine startup comparison included in each benchmark run's job summary

To run benchmarks locally:

```bash
# Criterion micro-benchmarks
cargo bench --bench startup

# Rust vs Python startup comparison (requires hyperfine + pip install cqlsh)
cargo build --release
scripts/bench_comparison.sh
```

## Running tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test cli_tests
```

## Project structure

```
src/
├── main.rs              # Entry point
├── cli.rs               # CLI argument parsing (clap v4)
├── config.rs            # cqlshrc parsing & merged configuration
└── shell_completions.rs # Shell completion generation
tests/
└── cli_tests.rs         # CLI integration tests
docs/
└── plans/               # Design documents and sub-plans
```

## License

[MIT](LICENSE)

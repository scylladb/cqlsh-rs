# cqlsh-rs: High-Level Design

## Overview

**cqlsh-rs** is a Rust re-implementation of the Python [`cqlsh`](https://cassandra.apache.org/doc/latest/cassandra/tools/cqlsh.html) tool — the official interactive command-line shell for Apache Cassandra (and compatible databases such as ScyllaDB). The goal is to produce a self-contained, fast, single-binary replacement with no Python runtime dependency.

---

## Goals

1. **Feature parity** with the Python `cqlsh` for common day-to-day operations.
2. **Single binary** — no runtime dependency on Python or any interpreter.
3. **Cross-platform** — Linux, macOS, and Windows.
4. **Async I/O** — built on Tokio for efficient network operations.
5. **Pluggable drivers** — abstract the Cassandra driver layer so both `cassandra-rs` / `cdrs-tokio` and the `scylla` crate can be used.

---

## Non-Goals (v0 / v1)

- Full parity with every edge-case behaviour of the Python tool.
- A graphical or web-based UI.
- Bundled Cassandra server.

---

## Feature Breakdown

### Phase 1 — Bootstrap (MVP)

| Feature | Details |
|---|---|
| Project scaffolding | `Cargo.toml`, workspace layout, CI skeleton |
| CLI argument parsing | Host, port, keyspace, username/password, `--execute`/`-e`, `--file`/`-f` |
| Cassandra connection | Plain TCP connection using the CQL binary protocol (v4/v5) |
| Basic REPL loop | Read a line, send to Cassandra, print raw response |
| Semicolon-terminated statements | Buffer multi-line input until `;` is found |
| Quit / exit commands | `QUIT`, `EXIT`, `Ctrl-D` |

### Phase 2 — Usable Shell

| Feature | Details |
|---|---|
| Line-editing & history | `rustyline`-based editing with persistent `~/.cqlsh_history` |
| Tabular result display | `comfy-table` or `prettytable-rs` with column type awareness |
| Pagination | `--no-pager`, page size config, interactive `--More--` prompt |
| Error display | Server-side errors rendered with error code and message |
| `USE <keyspace>` tracking | Update prompt to `cqlsh:<keyspace>` |
| `DESCRIBE` commands | `DESCRIBE KEYSPACES`, `DESCRIBE TABLES`, `DESCRIBE TABLE <t>` |
| `SOURCE` command | Execute statements from a `.cql` file |

### Phase 3 — Quality of Life

| Feature | Details |
|---|---|
| Tab completion | CQL keywords, keyspace names, table names, column names |
| Output formats | `--output-format ascii/json/csv` |
| `COPY TO / FROM` | Export and import CSV data |
| Coloured output | Syntax highlighting in queries, colour-coded result headers |
| SSL/TLS | `--ssl`, `--certfile`, `--userkey`, `--usercert` |
| Configuration file | `~/.cqlshrc` (INI format compatible with Python cqlsh) |
| `TRACING ON/OFF` | Request tracing from Cassandra |
| `CONSISTENCY` | Set/display the consistency level in session |

### Phase 4 — Advanced

| Feature | Details |
|---|---|
| Schema introspection helpers | `DESCRIBE FULL SCHEMA`, `DESCRIBE TYPE`, `DESCRIBE INDEX` |
| Token-aware routing display | Show which replicas will handle a query |
| Batch / scripting mode | `--execute` and `--file` with proper exit codes |
| Metrics | Optional latency/throughput display after each query |
| Plugin hooks | Pre/post-query hooks for custom tooling |

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                         cqlsh-rs                         │
│                                                          │
│  ┌────────────┐   ┌──────────────┐   ┌───────────────┐  │
│  │  CLI Args  │   │  Config File │   │   Env Vars    │  │
│  └─────┬──────┘   └──────┬───────┘   └──────┬────────┘  │
│        └─────────────────┼──────────────────┘           │
│                          ▼                               │
│               ┌─────────────────────┐                   │
│               │   Session Manager   │                   │
│               │  (auth, keyspace,   │                   │
│               │   consistency, …)   │                   │
│               └──────────┬──────────┘                   │
│                          │                               │
│          ┌───────────────┼───────────────┐               │
│          ▼               ▼               ▼               │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐     │
│  │  REPL Loop   │ │ Batch/Script │ │  Execute -e  │     │
│  │ (rustyline)  │ │  (--file)    │ │  (one-shot)  │     │
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘     │
│         └────────────────┼────────────────┘             │
│                          ▼                               │
│               ┌─────────────────────┐                   │
│               │   Statement Parser  │                   │
│               │  (multi-line buf,   │                   │
│               │   special cmds)     │                   │
│               └──────────┬──────────┘                   │
│                          │                               │
│          ┌───────────────┼───────────────┐               │
│          ▼               ▼               ▼               │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐     │
│  │ Driver Layer │ │  Formatter   │ │  Completer   │     │
│  │  (scylla /  │ │ (table/json/ │ │ (keywords +  │     │
│  │  cdrs-tokio) │ │    csv)      │ │  schema)     │     │
│  └──────────────┘ └──────────────┘ └──────────────┘     │
└──────────────────────────────────────────────────────────┘
```

### Key Modules

| Module | Responsibility |
|---|---|
| `main.rs` | Entry point: parse CLI, bootstrap session, choose run-mode |
| `config.rs` | Load and merge `~/.cqlshrc`, CLI args, env vars |
| `session.rs` | Manage Cassandra connection, auth, keyspace, consistency |
| `repl.rs` | Interactive REPL loop using `rustyline` |
| `runner.rs` | Non-interactive execution (file / `-e`) |
| `parser.rs` | Buffer multi-line input, detect statement boundaries, recognise built-in commands |
| `commands/` | One file per built-in command (`describe.rs`, `copy.rs`, `source.rs`, …) |
| `formatter.rs` | Render result sets as table, JSON, or CSV |
| `completer.rs` | Tab-completion based on CQL keywords + live schema |
| `driver/` | Thin trait + implementation(s) for the Cassandra driver |
| `error.rs` | Unified error type |

---

## Dependency Candidates

| Crate | Purpose |
|---|---|
| [`scylla`](https://crates.io/crates/scylla) | Async Cassandra/ScyllaDB driver (preferred) |
| [`clap`](https://crates.io/crates/clap) | CLI argument parsing |
| [`rustyline`](https://crates.io/crates/rustyline) | Line editing, history, completion |
| [`comfy-table`](https://crates.io/crates/comfy-table) | Terminal table rendering |
| [`serde_json`](https://crates.io/crates/serde_json) | JSON output |
| [`tokio`](https://crates.io/crates/tokio) | Async runtime |
| [`ini`](https://crates.io/crates/ini) | Parse `~/.cqlshrc` |
| [`owo-colors`](https://crates.io/crates/owo-colors) | Coloured terminal output |
| [`csv`](https://crates.io/crates/csv) | CSV output / COPY TO |
| [`anyhow`](https://crates.io/crates/anyhow) | Error handling ergonomics |
| [`tracing`](https://crates.io/crates/tracing) + [`tracing-subscriber`](https://crates.io/crates/tracing-subscriber) | Structured logging / diagnostics |

---

## Repository Layout (target)

```
cqlsh-rs/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── docs/
│   └── plans/
│       └── high-level-design.md   ← this file
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── session.rs
│   ├── repl.rs
│   ├── runner.rs
│   ├── parser.rs
│   ├── formatter.rs
│   ├── completer.rs
│   ├── error.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── describe.rs
│   │   ├── copy.rs
│   │   └── source.rs
│   └── driver/
│       ├── mod.rs
│       └── scylla.rs
└── tests/
    ├── integration/
    └── unit/
```

---

## Implementation Order

1. **`Cargo.toml` + `src/main.rs`** — bare-bones binary that prints "Hello from cqlsh-rs".
2. **`config.rs` + `clap` wiring** — accept host/port/user/password arguments.
3. **`session.rs`** — establish a Cassandra connection and run a hard-coded `SELECT now() FROM system.local`.
4. **`repl.rs`** — readline loop, multi-line buffering, semicolon detection.
5. **`formatter.rs`** — tabular output for `SELECT` results.
6. **`parser.rs` + built-in commands** — `USE`, `QUIT`, `DESCRIBE`, `SOURCE`.
7. **`completer.rs`** — keyword and schema-aware tab-completion.
8. **`config.rs` (cqlshrc)** — persist settings across sessions.
9. **`copy.rs`** — COPY TO / FROM CSV.
10. **End-to-end integration tests** — spin up a Cassandra/Scylla container, run queries, assert output.

---

## Compatibility Target

| Attribute | Target |
|---|---|
| Cassandra versions | 3.x, 4.x, 5.x |
| ScyllaDB versions | 5.x, 6.x |
| CQL protocol | v4 (default), v5 (optional) |
| Minimum Rust edition | 2021 |
| Minimum Rust toolchain | stable (latest) |

---

## Open Questions

- Should the driver abstraction support `cdrs-tokio` in addition to `scylla`, or should we commit to `scylla` only?
- Do we want to ship pre-built binaries via GitHub Releases from day one?
- Should `COPY FROM` be implemented in the MVP or deferred to Phase 3?
- How closely should `~/.cqlshrc` parsing mirror the Python implementation (same section names, same defaults)?

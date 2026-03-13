# cqlsh-rs: High-Level Design

## Overview

**cqlsh-rs** is a Rust re-implementation of [**scylla-cqlsh**](https://github.com/scylladb/scylla-cqlsh) — the ScyllaDB-maintained fork of the Python CQL shell. The goal is to produce a self-contained, fast, single-binary replacement that is feature-compatible with scylla-cqlsh but requires no Python runtime.

scylla-cqlsh is itself forked from the Apache Cassandra `cqlsh` tool and is kept in sync with ScyllaDB's release cycle. It uses the [scylla-driver](https://github.com/scylladb/python-driver) (the ScyllaDB fork of the Cassandra Python driver). cqlsh-rs will use the [`scylla`](https://crates.io/crates/scylla) Rust crate as its driver — the Rust-native equivalent built and maintained by ScyllaDB.

> **Primary target: ScyllaDB (5.x, 6.x)**. Apache Cassandra (3.x, 4.x, 5.x) is a secondary compatibility target.

---

## Reference Implementation

| Property | Value |
|---|---|
| Reference codebase | <https://github.com/scylladb/scylla-cqlsh> |
| Reference main script | `bin/cqlsh.py` |
| Reference library | `pylib/cqlshlib/` |
| Python driver used | `scylla-driver` (ScyllaDB fork of `cassandra-driver`) |
| Distributed as | PyPI package `scylla-cqlsh`, Docker image `scylladb/scylla-cqlsh` |

All behavioural decisions should default to what scylla-cqlsh does, not what the upstream Cassandra cqlsh does. Where the two differ, scylla-cqlsh wins.

---

## Goals

1. **Feature parity with scylla-cqlsh** for common day-to-day ScyllaDB operations.
2. **Single binary** — no runtime dependency on Python or any interpreter.
3. **Cross-platform** — Linux, macOS, and Windows.
4. **Async I/O** — built on Tokio and the `scylla` crate for efficient network operations.
5. **ScyllaDB-first** — ScyllaDB-specific features and system tables take priority; Cassandra compatibility is best-effort.

---

## Non-Goals (v0 / v1)

- Full parity with every edge-case behaviour of scylla-cqlsh.
- A graphical or web-based UI.
- Bundled ScyllaDB/Cassandra server.
- Maintaining a separate `cdrs-tokio` driver backend.

---

## ScyllaDB-Specific Behaviours to Replicate

These are features present in scylla-cqlsh that differ from or extend the upstream Cassandra cqlsh:

### Version Detection & Prompt Banner

scylla-cqlsh queries `SELECT * FROM system.versions WHERE key = 'local'` on connect. If the table exists and returns a `version` column, the node is ScyllaDB and the banner reads:

```
[cqlsh X.Y | Scylla A.B.C | CQL spec D.E.F | Native protocol vN]
```

If the table does not exist (Cassandra node), it falls back to querying `system.local`:

```
[cqlsh X.Y | Cassandra A.B.C | CQL spec D.E.F | Native protocol vN]
```

cqlsh-rs must implement the same two-step detection.

### Unix Domain Socket Support

scylla-cqlsh accepts a Unix socket path as the host argument (e.g. `/var/run/scylla/cql.sock`) and detects it via `stat(2)`. The driver is then configured to connect via `UnixSocketEndPoint`. cqlsh-rs must support the same by detecting a socket path on the CLI and passing it to the `scylla` crate.

### `--no-compression` Flag

scylla-cqlsh exposes `--no-compression` to disable the LZ4 compression that the driver enables by default. cqlsh-rs must expose the same flag and pass it through to the driver.

### `CQLSH_PROMPT` Environment Variable

scylla-cqlsh reads `$CQLSH_PROMPT` at startup and prepends its value to the default prompt lines:

```
$CQLSH_PROMPT
cqlsh>
```

cqlsh-rs must honour the same variable.

### Pluggable Auth Provider via `~/.cassandra/cqlshrc`

scylla-cqlsh supports configuring any Cassandra auth provider (not just `PlainTextAuthProvider`) through the `[auth_provider]` section of `cqlshrc`:

```ini
[auth_provider]
module = cassandra.auth
classname = SomeSaslAuthProvider
username = alice
password = secret
```

Credentials in a separate `~/.cassandra/credentials` file override those in `cqlshrc`. cqlsh-rs should model the same two-file separation and support configuring alternative auth mechanisms via `cqlshrc`.

### SSL/TLS Configuration

scylla-cqlsh reads SSL settings from `[ssl]` and `[certfiles]` sections of `cqlshrc` and from environment variables (`SSL_CERTFILE`, `SSL_VALIDATE`, `SSL_CHECK_HOSTNAME`, `SSL_VERSION`). Relevant options:

| cqlshrc key | Env var | Meaning |
|---|---|---|
| `[ssl] certfile` | `SSL_CERTFILE` | CA certificate file |
| `[ssl] validate` | `SSL_VALIDATE` | Verify server cert (`true`/`false`) |
| `[ssl] check_hostname` | `SSL_CHECK_HOSTNAME` | Verify hostname in cert |
| `[ssl] userkey` | — | Client private key |
| `[ssl] usercert` | — | Client certificate |
| `[certfiles] <host>` | — | Per-host CA cert |

The TLS version is auto-negotiated (explicit version strings are accepted but produce a warning).

### ScyllaDB System Keyspaces

scylla-cqlsh treats the following keyspaces as non-alterable (no `ALTER KEYSPACE` completion offered):

```
system, system_schema, system_views, system_virtual_schema,
system_distributed_everywhere, system_replicated_keys
```

These are all ScyllaDB-specific; the upstream Cassandra cqlsh only lists `system` and `system_schema`. cqlsh-rs must use the ScyllaDB list.

### ScyllaDB CQL Extensions

ScyllaDB extends CQL with features that scylla-cqlsh surfaces in tab-completion and `DESCRIBE` output. These include:

- **CDC (Change Data Capture)** — `cdc = {'enabled': true}` table option
- **ScyllaDB compaction strategies** — `IncrementalCompactionStrategy` in addition to the standard set
- **`BYPASS CACHE`** — query modifier for reads that should bypass the row cache
- **`USING TIMEOUT`** — per-statement timeout override
- **`system_distributed_everywhere` / `system_replicated_keys`** — ScyllaDB-specific system keyspaces

### `--driver-debug` Flag

scylla-cqlsh exposes `--driver-debug` to enable verbose logging from the underlying Python driver. cqlsh-rs should support an equivalent flag that activates `RUST_LOG=scylla=debug` (or similar) output.

### `CQL_HISTORY` Environment Variable

scylla-cqlsh checks `$CQL_HISTORY` to override the default history file location (`~/.cassandra/cqlsh_history`). cqlsh-rs must honour the same variable.

### Default Config Directory

scylla-cqlsh stores config in `~/.cassandra/` (not `~/.cqlsh/`):

| File | Purpose |
|---|---|
| `~/.cassandra/cqlshrc` | Main configuration |
| `~/.cassandra/credentials` | Auth credentials (separate from cqlshrc) |
| `~/.cassandra/cqlsh_history` | Command history |

cqlsh-rs must use the same paths for drop-in compatibility.

---

## Feature Breakdown

> **How to read the Status blocks**
>
> Each phase has a **Status** table immediately below its heading. Update this table as work progresses:
>
> | Field | Meaning |
> |---|---|
> | Progress | ASCII progress bar + count. Fill one `█` per completed item; leave `░` for remaining ones. |
> | Status | 🔴 Not Started · 🟡 In Progress · 🟢 Complete |
> | First commit | Date (YYYY-MM-DD) of the first commit touching code for this phase. |
> | Completed | Date (YYYY-MM-DD) the last feature in this phase was merged and verified. |
> | Time spent | Human estimate of total active development time (e.g. `~8 h`, `~3 days`). |
>
> **Progress bar key:** `░` = not started, `█` = done. Bar is always 10 characters wide; one character ≈ 10% regardless of the exact item count.
> Example at 3/7 done → `████░░░░░░ 3 / 7 items complete`

### Phase 1 — Bootstrap (MVP)

#### Status

| Field | Value |
|---|---|
| Progress | `░░░░░░░░░░` 0 / 7 items complete |
| Status | 🔴 Not Started |
| First commit | — |
| Completed | — |
| Time spent | — |

| Feature | scylla-cqlsh reference | Details |
|---|---|---|
| Project scaffolding | — | `Cargo.toml`, workspace layout, CI skeleton |
| CLI argument parsing | `parser.add_option(…)` | Host, port, keyspace, username/password, `--execute`/`-e`, `--file`/`-f`, `--no-compression` |
| ScyllaDB connection | `cassandra.cluster.Cluster(…)` | TCP connection via `scylla` crate using CQL native protocol v4/v5 |
| Version detection | `get_scylla_version()` | Query `system.versions`; fall back to `system.local`; print correct banner |
| Basic REPL loop | `Shell.cmdloop()` | Read a line, send to ScyllaDB, print raw response |
| Semicolon-terminated statements | `onecmd()` | Buffer multi-line input until `;` is found |
| Quit / exit commands | `do_quit()` | `QUIT`, `EXIT`, `Ctrl-D` |

### Phase 2 — Usable Shell

#### Status

| Field | Value |
|---|---|
| Progress | `░░░░░░░░░░` 0 / 9 items complete |
| Status | 🔴 Not Started |
| First commit | — |
| Completed | — |
| Time spent | — |

| Feature | scylla-cqlsh reference | Details |
|---|---|---|
| Line-editing & history | `readline` / `CQL_HISTORY` | `rustyline`-based editing with `~/.cassandra/cqlsh_history`; respect `$CQL_HISTORY` |
| `CQLSH_PROMPT` support | `Shell.custom_prompt` | Prefix prompt lines with `$CQLSH_PROMPT` |
| Tabular result display | `displaying.py` | `comfy-table` with column type awareness; colour partition/clustering/static key columns differently |
| Pagination | `Shell.use_paging` | `--no-pager`, page size config, interactive `--More--` prompt |
| Error display | `Shell.printerr()` | Server-side errors rendered with error code and message |
| `USE <keyspace>` tracking | `do_use()` | Update prompt to `cqlsh:<keyspace>` |
| `DESCRIBE` commands | `do_desc()` | `DESCRIBE KEYSPACES`, `DESCRIBE TABLES`, `DESCRIBE TABLE <t>` |
| `SOURCE` command | `do_source()` | Execute statements from a `.cql` file |
| Unix socket support | `is_unix_socket()` | Detect socket path, connect via Unix domain socket |

### Phase 3 — Quality of Life

#### Status

| Field | Value |
|---|---|
| Progress | `░░░░░░░░░░` 0 / 11 items complete |
| Status | 🔴 Not Started |
| First commit | — |
| Completed | — |
| Time spent | — |

| Feature | scylla-cqlsh reference | Details |
|---|---|---|
| Tab completion | `cql3handling.py` | CQL keywords, keyspace names, table names, column names; ScyllaDB keywords included |
| Output formats | `--output-format` | `ascii`/`json`/`csv` |
| `COPY TO / FROM` | `copyutil.py` | Export and import CSV data |
| Coloured output | `displaying.py` | ANSI colours; partition key = red, clustering key = cyan, static = white |
| SSL/TLS | `sslhandling.py` | `--ssl`, `[ssl]` / `[certfiles]` in cqlshrc, env var overrides |
| Configuration file | `cqlshrc` parsing | `~/.cassandra/cqlshrc` (INI format); `~/.cassandra/credentials` for auth credentials |
| Pluggable auth provider | `authproviderhandling.py` | `[auth_provider]` section in cqlshrc; credentials file overrides |
| `TRACING ON/OFF` | `do_tracing()` | Request tracing from ScyllaDB |
| `CONSISTENCY` | `do_consistency()` | Set/display the consistency level in session |
| `--driver-debug` | `--driver-debug` | Enable verbose driver logging |
| `--no-compression` | `--no-compression` | Disable LZ4 compression |

### Phase 4 — Advanced

#### Status

| Field | Value |
|---|---|
| Progress | `░░░░░░░░░░` 0 / 10 items complete |
| Status | 🔴 Not Started |
| First commit | — |
| Completed | — |
| Time spent | — |

| Feature | scylla-cqlsh reference | Details |
|---|---|---|
| Full `DESCRIBE` suite | `do_desc()` | `DESCRIBE FULL SCHEMA`, `DESCRIBE TYPE`, `DESCRIBE INDEX`, `DESCRIBE MATERIALIZED VIEW` |
| ScyllaDB CDC in schema DDL | `cql3handling.py` | Recognise and complete `cdc = {…}` table option |
| `BYPASS CACHE` / `USING TIMEOUT` | `cql3handling.py` | Complete ScyllaDB CQL extensions |
| Token-aware replica display | `show_replicas()` | `SHOW REPLICAS <token>` command |
| `SHOW` commands | `do_show()` | `SHOW VERSION`, `SHOW HOST`, `SHOW SESSION <id>` |
| Batch / scripting mode | exit codes | `--execute` and `--file` with proper exit codes |
| `SERIAL CONSISTENCY` | `do_serial_consistency()` | Set/display the serial consistency level |
| `EXPAND ON/OFF` | `do_expand()` | Vertical result display |
| `PAGING ON/OFF/<n>` | `do_paging()` | Toggle paging or set page size interactively |
| Coverage mode | `--coverage` | Collect coverage data (useful for ScyllaDB CI) |

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                          cqlsh-rs                            │
│                                                              │
│  ┌────────────┐   ┌──────────────┐   ┌─────────────────┐    │
│  │  CLI Args  │   │  cqlshrc     │   │   Env Vars      │    │
│  │  (clap)    │   │  ~/.cassandra│   │ CQLSH_PROMPT    │    │
│  │            │   │  /cqlshrc    │   │ CQL_HISTORY     │    │
│  │            │   │  /credentials│   │ SSL_CERTFILE …  │    │
│  └─────┬──────┘   └──────┬───────┘   └──────┬──────────┘    │
│        └─────────────────┼──────────────────┘               │
│                          ▼                                   │
│               ┌─────────────────────┐                       │
│               │   Session Manager   │                       │
│               │  (auth, keyspace,   │                       │
│               │   consistency,      │                       │
│               │   Scylla version    │                       │
│               │   detection)        │                       │
│               └──────────┬──────────┘                       │
│                          │                                   │
│          ┌───────────────┼───────────────┐                   │
│          ▼               ▼               ▼                   │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐         │
│  │  REPL Loop   │ │ Batch/Script │ │  Execute -e  │         │
│  │ (rustyline)  │ │  (--file)    │ │  (one-shot)  │         │
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘         │
│         └────────────────┼────────────────┘                 │
│                          ▼                                   │
│               ┌─────────────────────┐                       │
│               │   Statement Parser  │                       │
│               │  (multi-line buf,   │                       │
│               │   special cmds,     │                       │
│               │   comment stripping)│                       │
│               └──────────┬──────────┘                       │
│                          │                                   │
│          ┌───────────────┼───────────────┐                   │
│          ▼               ▼               ▼                   │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐         │
│  │ scylla crate │ │  Formatter   │ │  Completer   │         │
│  │ (TCP + Unix  │ │ (table/json/ │ │ (CQL keywords│         │
│  │  socket,     │ │    csv,      │ │  + ScyllaDB  │         │
│  │  SSL, auth)  │ │  ANSI color) │ │  extensions) │         │
│  └──────────────┘ └──────────────┘ └──────────────┘         │
└──────────────────────────────────────────────────────────────┘
```

### Key Modules

| Module | Responsibility |
|---|---|
| `main.rs` | Entry point: parse CLI, bootstrap session, choose run-mode |
| `config.rs` | Load and merge `~/.cassandra/cqlshrc`, `~/.cassandra/credentials`, CLI args, env vars |
| `session.rs` | Manage ScyllaDB connection (TCP + Unix socket), Scylla version detection, auth, keyspace, consistency |
| `repl.rs` | Interactive REPL loop using `rustyline`; `CQLSH_PROMPT` support |
| `runner.rs` | Non-interactive execution (file / `-e`) with proper exit codes |
| `parser.rs` | Buffer multi-line input, strip comments, detect statement boundaries, recognise built-in commands |
| `commands/` | One file per built-in command (`describe.rs`, `copy.rs`, `source.rs`, `show.rs`, …) |
| `formatter.rs` | Render result sets as table/JSON/CSV; ANSI colour for partition/clustering/static columns |
| `completer.rs` | Tab-completion: CQL keywords (including ScyllaDB extensions) + live schema |
| `ssl.rs` | SSL/TLS context builder; reads `[ssl]` / `[certfiles]` from cqlshrc and env vars |
| `auth.rs` | Auth provider construction; reads `[auth_provider]` from cqlshrc; credentials file support |
| `error.rs` | Unified error type |

---

## Dependency Candidates

| Crate | Purpose |
|---|---|
| [`scylla`](https://crates.io/crates/scylla) | Async ScyllaDB/Cassandra driver — **primary driver, not abstracted away** |
| [`clap`](https://crates.io/crates/clap) | CLI argument parsing |
| [`rustyline`](https://crates.io/crates/rustyline) | Line editing, history (`CQL_HISTORY`), completion |
| [`comfy-table`](https://crates.io/crates/comfy-table) | Terminal table rendering |
| [`serde_json`](https://crates.io/crates/serde_json) | JSON output |
| [`tokio`](https://crates.io/crates/tokio) | Async runtime |
| [`configparser`](https://crates.io/crates/configparser) | Parse `~/.cassandra/cqlshrc` (INI format); chosen because it matches Python's `configparser` section/key semantics that scylla-cqlsh relies on |
| [`owo-colors`](https://crates.io/crates/owo-colors) | ANSI colour output |
| [`csv`](https://crates.io/crates/csv) | CSV output / COPY TO/FROM |
| [`anyhow`](https://crates.io/crates/anyhow) | Error handling ergonomics |
| [`rustls`](https://crates.io/crates/rustls) + [`rustls-pemfile`](https://crates.io/crates/rustls-pemfile) | TLS; CA cert / client cert loading |
| [`tracing`](https://crates.io/crates/tracing) + [`tracing-subscriber`](https://crates.io/crates/tracing-subscriber) | Structured logging / `--driver-debug` support |

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
│   ├── ssl.rs
│   ├── auth.rs
│   ├── error.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── describe.rs
│   │   ├── copy.rs
│   │   ├── show.rs
│   │   └── source.rs
└── tests/
    ├── integration/   ← spin up ScyllaDB via Docker
    └── unit/
```

---

## Implementation Order

1. **`Cargo.toml` + `src/main.rs`** — bare-bones binary that prints version banner.
2. **`config.rs` + `clap` wiring** — accept all scylla-cqlsh CLI flags; load `~/.cassandra/cqlshrc`.
3. **`session.rs`** — connect to ScyllaDB, detect Scylla version via `system.versions`, print banner.
4. **`repl.rs`** — readline loop, multi-line buffering, semicolon detection, `$CQLSH_PROMPT`.
5. **`formatter.rs`** — tabular output with ANSI key-column colouring.
6. **`parser.rs` + built-in commands** — `USE`, `QUIT`, `DESCRIBE`, `SOURCE`, `SHOW`.
7. **`ssl.rs`** — TLS support from cqlshrc / env vars.
8. **`auth.rs`** — pluggable auth provider from cqlshrc + credentials file.
9. **`completer.rs`** — CQL keywords + ScyllaDB extensions + live schema.
10. **`copy.rs`** — COPY TO / FROM CSV.
11. **End-to-end integration tests** — spin up a ScyllaDB container, run queries, assert output.

---

## Compatibility Target

| Attribute | Target |
|---|---|
| **Primary** ScyllaDB versions | 5.x, 6.x (and ScyllaDB Enterprise equivalents) |
| **Secondary** Cassandra versions | 3.x, 4.x, 5.x (best-effort) |
| CQL protocol | v4 (default), v5 (optional) |
| Minimum Rust edition | 2021 |
| Minimum Rust toolchain | stable (latest) |
| Config file compatibility | `~/.cassandra/cqlshrc` and `~/.cassandra/credentials` — same paths and section names as scylla-cqlsh |

---

## Open Questions

- Should Unix socket support use Tokio's `UnixStream` directly or go through the `scylla` crate's Unix socket connection option?
- Should `COPY FROM` be implemented in the MVP or deferred to Phase 3?
- How closely should ScyllaDB-specific CQL extension completions track scylla-cqlsh's `cql3handling.py`?
- Should we ship pre-built binaries via GitHub Releases from day one (mirrors the PyPI / Docker distribution of scylla-cqlsh)?
- Should the integration test suite reuse the same ScyllaDB Docker image that scylla-cqlsh's CI uses (`scylladb/scylla:latest`)?

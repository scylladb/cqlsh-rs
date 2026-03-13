# cqlsh-rs: Comprehensive Design & Master Plan

## Mission Statement

**cqlsh-rs** is a ground-up Rust re-implementation of the Python [`cqlsh`](https://cassandra.apache.org/doc/latest/cassandra/tools/cqlsh.html) — the official interactive CQL shell for Apache Cassandra and compatible databases (ScyllaDB, Amazon Keyspaces, Astra DB). The project targets **100% command-line and configuration compatibility** with the original Python cqlsh, delivering a zero-dependency single binary that is faster, more reliable, and fully tested.

---

## Table of Contents

1. [Goals & Non-Goals](#goals--non-goals)
2. [100% Compatibility Matrix](#100-compatibility-matrix)
3. [Architecture Overview](#architecture-overview)
4. [Phased Implementation Plan](#phased-implementation-plan)
5. [Sub-Plan Index](#sub-plan-index)
6. [Testing Strategy](#testing-strategy)
7. [Benchmarking Strategy](#benchmarking-strategy)
8. [Required Skills & Team Competencies](#required-skills--team-competencies)
9. [Skills Development Plan](#skills-development-plan)
10. [Risk Register](#risk-register)
11. [Open Questions & Decisions](#open-questions--decisions)

---

## Goals & Non-Goals

### Goals

| # | Goal | Measure of Success |
|---|------|-------------------|
| G1 | **100% CLI compatibility** | Every flag, argument, and environment variable from Python cqlsh is accepted and behaves identically |
| G2 | **100% configuration compatibility** | `~/.cqlshrc` files from existing Python cqlsh installations work without modification |
| G3 | **100% tab-completion parity** | Keywords, keyspace/table/column names, CQL types, and function names complete identically |
| G4 | **100% shell command parity** | Every interactive command (DESCRIBE, COPY, CONSISTENCY, TRACING, EXPAND, PAGING, etc.) works identically |
| G5 | **Single binary, zero runtime deps** | `cargo build --release` produces one static binary; no Python, Java, or shared libs needed |
| G6 | **Cross-platform** | Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64) |
| G7 | **Fully tested** | Unit tests, integration tests, compatibility tests, property-based tests; >90% coverage |
| G8 | **Benchmarked** | Reproducible benchmarks comparing startup time, query throughput, COPY performance, memory usage vs Python cqlsh |
| G9 | **Async I/O** | Built on Tokio for efficient network operations and concurrent COPY operations |
| G10 | **Drop-in replacement** | Users can alias `cqlsh` to `cqlsh-rs` with no workflow changes |

### Non-Goals (v1)

- Full parity with every undocumented quirk/bug of Python cqlsh (but document divergences)
- Graphical or web-based UI
- Bundled Cassandra/Scylla server
- CQL language server protocol (LSP) — future consideration
- Acting as a general-purpose database client (PostgreSQL, MySQL, etc.)

---

## 100% Compatibility Matrix

### Command-Line Arguments (Complete)

Every argument from `cqlsh --help` must be supported:

| Flag | Short | Description | Priority |
|------|-------|-------------|----------|
| `--version` | | Print cqlsh version | P1 |
| `--color` | `-C` | Force colored output | P2 |
| `--no-color` | | Disable colored output | P2 |
| `--browser` | | Browser for `HELP` (unused in modern cqlsh) | P4 |
| `--ssl` | | Enable SSL/TLS connection | P1 |
| `--no-file-io` | | Disable file I/O commands (COPY, SOURCE, CAPTURE) | P3 |
| `--debug` | | Show additional debug info | P2 |
| `--coverage` | | Collect coverage (internal, may skip) | P4 |
| `--execute` | `-e` | Execute a CQL statement and exit | P1 |
| `--file` | `-f` | Execute statements from a file | P1 |
| `--keyspace` | `-k` | Default keyspace | P1 |
| `--username` | `-u` | Authentication username | P1 |
| `--password` | `-p` | Authentication password | P1 |
| `--connect-timeout` | | Connection timeout in seconds | P1 |
| `--request-timeout` | | Per-request timeout in seconds | P1 |
| `--tty` | | Force TTY mode | P3 |
| `--encoding` | | Set character encoding (default: utf-8) | P2 |
| `--cqlshrc` | | Path to cqlshrc file | P1 |
| `--cqlversion` | | CQL version to use | P2 |
| `--protocol-version` | | Native protocol version | P2 |
| `[host]` | | Positional: contact point hostname | P1 |
| `[port]` | | Positional: native transport port | P1 |

### Environment Variables

| Variable | Description | Priority |
|----------|-------------|----------|
| `CQLSH_HOST` | Default host | P1 |
| `CQLSH_PORT` | Default port | P1 |
| `CQLSH_NO_BUNDLED` | Skip bundled driver (N/A for Rust, accept and ignore) | P4 |
| `SSL_CERTFILE` | SSL certificate file path | P2 |
| `SSL_VALIDATE` | Enable/disable certificate validation | P2 |
| `CQLSH_DEFAULT_CONNECT_TIMEOUT_SECONDS` | Default connect timeout | P2 |
| `CQLSH_DEFAULT_REQUEST_TIMEOUT_SECONDS` | Default request timeout | P2 |

### Shell Commands (Complete)

| Command | Description | Priority | Sub-Plan |
|---------|-------------|----------|----------|
| `HELP` / `?` | Show help for commands | P1 | [commands.md](07-builtin-commands.md) |
| `QUIT` / `EXIT` | Exit the shell | P1 | [commands.md](07-builtin-commands.md) |
| `USE <keyspace>` | Switch keyspace, update prompt | P1 | [commands.md](07-builtin-commands.md) |
| `DESCRIBE` / `DESC` | Schema introspection (see below) | P1 | [describe.md](07-builtin-commands.md) |
| `CONSISTENCY [level]` | Get/set consistency level | P1 | [commands.md](07-builtin-commands.md) |
| `SERIAL CONSISTENCY [level]` | Get/set serial consistency | P2 | [commands.md](07-builtin-commands.md) |
| `TRACING [ON\|OFF]` | Toggle query tracing | P1 | [commands.md](07-builtin-commands.md) |
| `EXPAND [ON\|OFF]` | Toggle vertical/expanded output | P2 | [commands.md](07-builtin-commands.md) |
| `PAGING [ON\|OFF\|<size>]` | Configure automatic paging | P2 | [commands.md](07-builtin-commands.md) |
| `LOGIN <username> [<password>]` | Re-authenticate without reconnecting | P2 | [commands.md](07-builtin-commands.md) |
| `SOURCE <file>` | Execute CQL file | P1 | [commands.md](07-builtin-commands.md) |
| `CAPTURE [<file>\|OFF]` | Capture output to file | P2 | [commands.md](07-builtin-commands.md) |
| `COPY TO` | Export table data to CSV | P2 | [copy.md](08-copy-to-from.md) |
| `COPY FROM` | Import CSV data into table | P2 | [copy.md](08-copy-to-from.md) |
| `SHOW VERSION` | Show cqlsh and Cassandra versions | P2 | [commands.md](07-builtin-commands.md) |
| `SHOW HOST` | Show connected host | P2 | [commands.md](07-builtin-commands.md) |
| `SHOW SESSION <uuid>` | Show tracing session details | P3 | [commands.md](07-builtin-commands.md) |
| `CLEAR` / `CLS` | Clear the terminal screen | P2 | [commands.md](07-builtin-commands.md) |

### DESCRIBE Sub-Commands

| Sub-Command | Example | Priority |
|-------------|---------|----------|
| `DESCRIBE CLUSTER` | `DESC CLUSTER` | P1 |
| `DESCRIBE SCHEMA` | `DESC SCHEMA` (full schema dump) | P1 |
| `DESCRIBE FULL SCHEMA` | `DESC FULL SCHEMA` | P2 |
| `DESCRIBE KEYSPACES` | `DESC KEYSPACES` | P1 |
| `DESCRIBE KEYSPACE [name]` | `DESC KEYSPACE system` | P1 |
| `DESCRIBE TABLES` | `DESC TABLES` | P1 |
| `DESCRIBE TABLE <name>` | `DESC TABLE users` | P1 |
| `DESCRIBE INDEX <name>` | `DESC INDEX idx_email` | P2 |
| `DESCRIBE MATERIALIZED VIEW` | `DESC MATERIALIZED VIEW mv_name` | P2 |
| `DESCRIBE TYPE <name>` | `DESC TYPE address` | P2 |
| `DESCRIBE TYPES` | `DESC TYPES` | P2 |
| `DESCRIBE FUNCTION <name>` | `DESC FUNCTION my_func` | P3 |
| `DESCRIBE FUNCTIONS` | `DESC FUNCTIONS` | P3 |
| `DESCRIBE AGGREGATE <name>` | `DESC AGGREGATE my_agg` | P3 |
| `DESCRIBE AGGREGATES` | `DESC AGGREGATES` | P3 |

### COPY TO/FROM Options (Complete)

| Option | Default | Used In | Description |
|--------|---------|---------|-------------|
| `DELIMITER` | `','` | TO/FROM | Column separator character |
| `QUOTE` | `'"'` | TO/FROM | Quote character |
| `ESCAPE` | `'\'` | TO/FROM | Escape character inside quotes |
| `HEADER` | `false` | TO/FROM | First row is header |
| `NULL` | `''` | TO/FROM | String representation of NULL |
| `DATETIMEFORMAT` | `'%Y-%m-%d %H:%M:%S%z'` | TO/FROM | Timestamp format |
| `MAXATTEMPTS` | `5` | FROM | Max retry attempts per batch |
| `REPORTFREQUENCY` | `0.25` | TO/FROM | Progress report interval (seconds) |
| `DECIMALSEP` | `'.'` | TO/FROM | Decimal separator |
| `THOUSANDSSEP` | `''` | TO/FROM | Thousands separator |
| `BOOLSTYLE` | `'True,False'` | TO/FROM | Boolean representation |
| `NUMPROCESSES` | cores-1 | FROM | Parallel worker processes |
| `MAXBATCHSIZE` | `20` | FROM | Rows per batch insert |
| `MINBATCHSIZE` | `10` | FROM | Minimum batch size |
| `CHUNKSIZE` | `5000` | FROM | Rows read per chunk |
| `INGESTRATE` | `100000` | FROM | Max rows/sec to ingest |
| `MAXPARSEERRORS` | `-1` | FROM | Max parse errors before abort |
| `MAXINSERTERRORS` | `1000` | FROM | Max insert errors before abort |
| `ERRFILE` | | FROM | File to log error rows |
| `PREPAREDSTATEMENTS` | `true` | FROM | Use prepared statements |
| `TTL` | `3600` | FROM | TTL for imported rows |
| `ENCODING` | `'utf-8'` | TO/FROM | File encoding |
| `PAGESIZE` | `1000` | TO | Fetch page size |
| `PAGETIMEOUT` | `10` | TO | Page fetch timeout |
| `BEGINTOKEN` | | TO | Start token for range export |
| `ENDTOKEN` | | TO | End token for range export |
| `MAXREQUESTS` | `6` | TO | Concurrent page requests |
| `MAXOUTPUTSIZE` | `-1` | TO | Max rows to export |
| `FLOATPRECISION` | `5` | TO | Float decimal precision |
| `DOUBLEPRECISION` | `12` | TO | Double decimal precision |

### Configuration File: `~/.cqlshrc`

Full INI-format compatibility with Python cqlsh:

```ini
[authentication]
username = cassandra
password = cassandra
keyspace = my_keyspace

[connection]
hostname = 127.0.0.1
port = 9042
factory = cqlshlib.ssl.ssl_transport_factory
timeout = 10
request_timeout = 10
connect_timeout = 5
client_timeout = 120

[ssl]
certfile = /path/to/ca-cert.pem
validate = true
userkey = /path/to/client-key.pem
usercert = /path/to/client-cert.pem
version = TLSv1_2

[ui]
color = on
datetimeformat = %Y-%m-%d %H:%M:%S%z
timezone = UTC
float_precision = 5
double_precision = 12
max_trace_wait = 10
encoding = utf-8
completekey = tab
browser =

[csv]
field_size_limit = 131072

[copy]
numprocesses = 4
maxattempts = 5
reportfrequency = 0.25

[copy-to]
pagesize = 1000
pagetimeout = 10
begintoken =
endtoken =
maxrequests = 6
maxoutputsize = -1
floatprecision = 5
doubleprecision = 12

[copy-from]
maxbatchsize = 20
minbatchsize = 10
chunksize = 5000
ingestrate = 100000
maxparseerrors = -1
maxinserterrors = 1000
preparedstatements = true
ttl = 3600

[tracing]
max_trace_wait = 10.0
```

### Tab Completion Rules

| Context | Completions Offered |
|---------|-------------------|
| Empty / start of statement | CQL keywords (`SELECT`, `INSERT`, `UPDATE`, `DELETE`, `CREATE`, `ALTER`, `DROP`, `USE`, `DESCRIBE`, `GRANT`, `REVOKE`, `LIST`, `BATCH`, `TRUNCATE`, etc.) + shell commands |
| After `SELECT ... FROM` | Keyspace-qualified and unqualified table names |
| After `INSERT INTO` | Table names |
| After `UPDATE` | Table names |
| After `DELETE FROM` | Table names |
| After table name (in SELECT etc.) | Column names for that table |
| After `USE` | Keyspace names |
| After `DESCRIBE` / `DESC` | `CLUSTER`, `SCHEMA`, `KEYSPACES`, `KEYSPACE`, `TABLES`, `TABLE`, `INDEX`, `TYPE`, `FUNCTION`, `AGGREGATE`, `MATERIALIZED VIEW` |
| After `DESCRIBE TABLE` | Table names |
| After `DESCRIBE KEYSPACE` | Keyspace names |
| After `CONSISTENCY` | Consistency levels (`ANY`, `ONE`, `TWO`, `THREE`, `QUORUM`, `ALL`, `LOCAL_QUORUM`, `EACH_QUORUM`, `SERIAL`, `LOCAL_SERIAL`, `LOCAL_ONE`) |
| After `CREATE TABLE` ... column type | CQL types (`text`, `int`, `bigint`, `float`, `double`, `boolean`, `timestamp`, `uuid`, `timeuuid`, `blob`, `inet`, `varint`, `decimal`, `counter`, `ascii`, `date`, `time`, `smallint`, `tinyint`, `duration`, `list`, `set`, `map`, `tuple`, `frozen`) |
| After `COPY` | Table names |
| After `COPY <table> TO/FROM` | File path completion |
| After `WITH` in DDL | Property names (e.g., `replication`, `compaction`, `gc_grace_seconds`, etc.) |
| After `WHERE` | Column names |
| After `AND` / `OR` in WHERE | Column names |
| After `ORDER BY` | Column names |
| After `LOGIN` | (no completion — accepts free text) |
| After `SOURCE` | File path completion |
| After `CAPTURE` | File path completion, `OFF` |

### Output Formatting Behaviors

| Behavior | Details |
|----------|---------|
| Tabular (default) | Aligned columns with `+`/`-`/`\|` borders |
| Expanded / Vertical | Each row shown as key-value block (triggered by `EXPAND ON`) |
| JSON output | `--output-format json` — each row as JSON object |
| CSV output | `--output-format csv` — RFC 4180 compliant |
| Tracing output | Session UUID, coordinator, duration, events table |
| NULL display | Empty by default, configurable |
| Timestamp format | Controlled by `datetimeformat` in cqlshrc |
| Float precision | Controlled by `float_precision` / `double_precision` |
| Boolean display | `True` / `False` |
| Blob display | `0x` prefixed hex string |
| UUID display | Standard UUID string |
| Collection display | `{...}` for sets/maps, `[...]` for lists |
| Frozen display | Same as underlying type |
| UDT display | `{field1: val1, field2: val2}` |
| Tuple display | `(val1, val2, val3)` |
| Pagination | `--More--` prompt, configurable page size |
| Color scheme | Keywords blue, strings green, numbers yellow, etc. |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              cqlsh-rs                                  │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                     Configuration Layer                          │  │
│  │  ┌────────────┐   ┌──────────────┐   ┌────────────────────────┐ │  │
│  │  │  CLI Args  │   │  ~/.cqlshrc  │   │   Environment Vars     │ │  │
│  │  │   (clap)   │   │    (ini)     │   │  (CQLSH_HOST, etc.)    │ │  │
│  │  └─────┬──────┘   └──────┬───────┘   └──────────┬─────────────┘ │  │
│  │        └─────────────────┼──────────────────────┘               │  │
│  │                          ▼                                       │  │
│  │               ┌─────────────────────┐                           │  │
│  │               │   MergedConfig      │                           │  │
│  │               │  (precedence:       │                           │  │
│  │               │   cli > env >       │                           │  │
│  │               │   cqlshrc > default)│                           │  │
│  │               └──────────┬──────────┘                           │  │
│  └──────────────────────────┼──────────────────────────────────────┘  │
│                             │                                         │
│  ┌──────────────────────────▼──────────────────────────────────────┐  │
│  │                     Session Manager                             │  │
│  │  ┌─────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐ │  │
│  │  │ Auth/Login  │ │  Keyspace  │ │Consistency │ │  Tracing   │ │  │
│  │  │  Manager    │ │  Tracker   │ │  Manager   │ │  Manager   │ │  │
│  │  └─────────────┘ └────────────┘ └────────────┘ └────────────┘ │  │
│  └──────────────────────────┬──────────────────────────────────────┘  │
│                             │                                         │
│         ┌───────────────────┼──────────────────────┐                  │
│         ▼                   ▼                      ▼                  │
│  ┌──────────────┐   ┌──────────────┐   ┌───────────────────┐         │
│  │  REPL Mode   │   │  Batch Mode  │   │   Execute Mode    │         │
│  │ (rustyline)  │   │  (--file)    │   │   (-e oneshot)    │         │
│  │  interactive │   │  scripting   │   │                   │         │
│  └──────┬───────┘   └──────┬───────┘   └──────────┬────────┘         │
│         └───────────────────┼──────────────────────┘                  │
│                             ▼                                         │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                    Statement Pipeline                            │ │
│  │  ┌─────────────┐ ┌──────────────┐ ┌───────────────────────────┐ │ │
│  │  │ Multi-line  │ │  Command     │ │   CQL Statement           │ │ │
│  │  │ Buffering   │→│  Router      │→│   Dispatcher              │ │ │
│  │  │ (;-detect)  │ │ (built-in    │ │  (to driver)              │ │ │
│  │  │             │ │  vs CQL)     │ │                           │ │ │
│  │  └─────────────┘ └──────────────┘ └───────────────────────────┘ │ │
│  └──────────────────────────┬──────────────────────────────────────┘ │
│                             │                                         │
│    ┌────────────────────────┼────────────────────────────┐            │
│    ▼                        ▼                            ▼            │
│  ┌──────────────┐   ┌──────────────┐           ┌──────────────┐      │
│  │   Driver     │   │  Formatter   │           │  Completer   │      │
│  │   Layer      │   │              │           │              │      │
│  │ ┌──────────┐ │   │ ┌──────────┐ │           │ ┌──────────┐ │      │
│  │ │  scylla  │ │   │ │  Table   │ │           │ │ Keywords │ │      │
│  │ │  driver  │ │   │ │  JSON    │ │           │ │ Schema   │ │      │
│  │ │          │ │   │ │  CSV     │ │           │ │ Columns  │ │      │
│  │ └──────────┘ │   │ │ Expanded │ │           │ │ Types    │ │      │
│  │ ┌──────────┐ │   │ │ Tracing  │ │           │ │ Files    │ │      │
│  │ │  driver  │ │   │ └──────────┘ │           │ └──────────┘ │      │
│  │ │  trait   │ │   │              │           │              │      │
│  │ └──────────┘ │   │ ┌──────────┐ │           │ ┌──────────┐ │      │
│  │              │   │ │ Colorizer│ │           │ │  Schema  │ │      │
│  │              │   │ └──────────┘ │           │ │  Cache   │ │      │
│  └──────────────┘   └──────────────┘           └──────────────┘      │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                     Built-in Commands                           │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │ │
│  │  │ DESCRIBE │ │   COPY   │ │  SOURCE  │ │ CAPTURE  │          │ │
│  │  │ (schema) │ │ (TO/FROM)│ │ (files)  │ │ (output) │          │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │ │
│  │  │  SHOW    │ │  LOGIN   │ │ PAGING   │ │  EXPAND  │          │ │
│  │  │ (info)   │ │ (auth)   │ │ (config) │ │ (format) │          │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │ │
│  └──────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

### Key Modules (Expanded)

| Module | Responsibility | Estimated Complexity |
|--------|---------------|---------------------|
| `main.rs` | Entry point: parse CLI, bootstrap session, choose run-mode | Low |
| `config.rs` | Load and merge `~/.cqlshrc`, CLI args, env vars with precedence rules | Medium |
| `session.rs` | Manage Cassandra connection pool, auth, keyspace tracking, consistency state | High |
| `repl.rs` | Interactive REPL loop using `rustyline`, prompt management, multi-line input | Medium |
| `runner.rs` | Non-interactive execution (file / `-e`), exit code management | Low |
| `parser.rs` | Multi-line buffering, semicolon detection, comment stripping, command routing | Medium |
| `formatter.rs` | Tabular, JSON, CSV, expanded output; type-aware formatting for all CQL types | High |
| `colorizer.rs` | Syntax highlighting for CQL, color-coded output | Medium |
| `completer.rs` | Context-aware tab-completion with schema introspection cache | High |
| `error.rs` | Unified error type with Cassandra error code mapping | Low |
| `types.rs` | CQL type system mapping and formatting rules | High |
| `commands/describe.rs` | Full DESCRIBE command tree (15+ sub-commands) | High |
| `commands/copy.rs` | COPY TO/FROM with all options, parallel workers, progress reporting | Very High |
| `commands/source.rs` | SOURCE file execution | Low |
| `commands/capture.rs` | CAPTURE output to file | Low |
| `commands/show.rs` | SHOW VERSION, SHOW HOST, SHOW SESSION | Low |
| `commands/login.rs` | LOGIN re-authentication | Medium |
| `commands/paging.rs` | PAGING configuration | Low |
| `commands/expand.rs` | EXPAND toggle | Low |
| `commands/consistency.rs` | CONSISTENCY and SERIAL CONSISTENCY management | Low |
| `commands/tracing_cmd.rs` | TRACING ON/OFF and trace display | Medium |
| `commands/help.rs` | HELP command with per-command help text | Low |
| `commands/clear.rs` | CLEAR/CLS terminal clearing | Low |
| `driver/mod.rs` | Driver trait definition | Low |
| `driver/scylla.rs` | scylla crate adapter implementation | High |

---

## Phased Implementation Plan

### Phase 1 — Bootstrap MVP (Weeks 1-3)

**Goal:** Minimal working shell that can connect and execute queries.

| Task | Description | Deliverable | Depends On |
|------|-------------|-------------|------------|
| 1.1 | Cargo workspace setup, CI (GitHub Actions), linting, formatting | `Cargo.toml`, `.github/workflows/ci.yml` | — |
| 1.2 | CLI argument parsing (full compatibility — all flags accepted) | `main.rs`, `config.rs` | 1.1 |
| 1.3 | Environment variable loading | `config.rs` | 1.2 |
| 1.4 | cqlshrc file parsing (INI format, all sections) | `config.rs` | 1.2 |
| 1.5 | Configuration merging with correct precedence | `config.rs` | 1.2, 1.3, 1.4 |
| 1.6 | Cassandra driver abstraction trait | `driver/mod.rs` | 1.1 |
| 1.7 | scylla crate driver implementation | `driver/scylla.rs` | 1.6 |
| 1.8 | Session establishment with auth | `session.rs` | 1.5, 1.7 |
| 1.9 | Basic REPL loop (read-line, no editing) | `repl.rs` | 1.8 |
| 1.10 | Multi-line statement buffering | `parser.rs` | 1.9 |
| 1.11 | QUIT / EXIT / Ctrl-D | `repl.rs` | 1.9 |
| 1.12 | Raw result printing (columns + rows) | `formatter.rs` | 1.8 |
| 1.13 | Execute mode (`-e`) | `runner.rs` | 1.8 |
| 1.14 | File mode (`-f`) | `runner.rs` | 1.8, 1.10 |
| 1.15 | `--version` flag | `main.rs` | 1.2 |

**Exit Criteria:** Can connect to Cassandra, run CQL queries, see results, exit cleanly. All CLI flags accepted (unimplemented ones produce warnings).

### Phase 2 — Usable Shell (Weeks 4-7)

**Goal:** A pleasant interactive experience with proper formatting and core commands.

| Task | Description | Deliverable | Depends On |
|------|-------------|-------------|------------|
| 2.1 | rustyline integration (line editing, history) | `repl.rs` | Phase 1 |
| 2.2 | Persistent `~/.cqlsh_history` | `repl.rs` | 2.1 |
| 2.3 | Prompt management (`cqlsh>`, `cqlsh:keyspace>`, `...>` for multi-line) | `repl.rs` | 2.1 |
| 2.4 | Type-aware tabular formatting (all CQL types) | `formatter.rs`, `types.rs` | Phase 1 |
| 2.5 | Pagination with `--More--` prompt | `formatter.rs` | 2.4 |
| 2.6 | Server error display with error codes | `formatter.rs` | Phase 1 |
| 2.7 | `USE <keyspace>` with prompt tracking | `commands/` | Phase 1 |
| 2.8 | `DESCRIBE KEYSPACES` | `commands/describe.rs` | Phase 1 |
| 2.9 | `DESCRIBE TABLES` | `commands/describe.rs` | Phase 1 |
| 2.10 | `DESCRIBE TABLE <t>` | `commands/describe.rs` | Phase 1 |
| 2.11 | `DESCRIBE KEYSPACE [name]` | `commands/describe.rs` | Phase 1 |
| 2.12 | `DESCRIBE CLUSTER` | `commands/describe.rs` | Phase 1 |
| 2.13 | `DESCRIBE SCHEMA` | `commands/describe.rs` | Phase 1 |
| 2.14 | `SOURCE <file>` command | `commands/source.rs` | Phase 1 |
| 2.15 | `CONSISTENCY [level]` command | `commands/consistency.rs` | Phase 1 |
| 2.16 | `SERIAL CONSISTENCY [level]` command | `commands/consistency.rs` | Phase 1 |
| 2.17 | `TRACING ON/OFF` command | `commands/tracing_cmd.rs` | Phase 1 |
| 2.18 | Trace output formatting | `commands/tracing_cmd.rs`, `formatter.rs` | 2.17 |
| 2.19 | `SHOW VERSION`, `SHOW HOST` | `commands/show.rs` | Phase 1 |
| 2.20 | `HELP` / `?` command with per-command help | `commands/help.rs` | Phase 1 |
| 2.21 | `CLEAR` / `CLS` command | `commands/clear.rs` | Phase 1 |
| 2.22 | SSL/TLS support (`--ssl`, cert options) | `session.rs`, `config.rs` | Phase 1 |

**Exit Criteria:** Daily-driver quality shell. Can replace Python cqlsh for interactive use with core commands.

### Phase 3 — Tab Completion & Output (Weeks 8-11)

**Goal:** Full tab completion parity and all output formats.

| Task | Description | Deliverable | Depends On |
|------|-------------|-------------|------------|
| 3.1 | CQL keyword completion | `completer.rs` | 2.1 |
| 3.2 | Schema metadata cache (keyspaces, tables, columns, types, functions) | `completer.rs` | Phase 1 |
| 3.3 | Keyspace name completion | `completer.rs` | 3.2 |
| 3.4 | Table name completion (context-aware) | `completer.rs` | 3.2 |
| 3.5 | Column name completion (context-aware) | `completer.rs` | 3.2 |
| 3.6 | CQL type completion | `completer.rs` | 3.1 |
| 3.7 | Consistency level completion | `completer.rs` | 3.1 |
| 3.8 | DESCRIBE sub-command completion | `completer.rs` | 3.1 |
| 3.9 | File path completion (SOURCE, CAPTURE, COPY) | `completer.rs` | 3.1 |
| 3.10 | Schema cache invalidation on DDL | `completer.rs` | 3.2 |
| 3.11 | JSON output format | `formatter.rs` | 2.4 |
| 3.12 | CSV output format | `formatter.rs` | 2.4 |
| 3.13 | Expanded/vertical output (EXPAND) | `formatter.rs` | 2.4 |
| 3.14 | `EXPAND ON/OFF` command | `commands/expand.rs` | 3.13 |
| 3.15 | `PAGING ON/OFF/<size>` command | `commands/paging.rs` | 2.5 |
| 3.16 | Syntax highlighting (colored CQL) | `colorizer.rs` | 2.1 |
| 3.17 | `--color` / `--no-color` flags | `colorizer.rs` | 3.16 |
| 3.18 | Color-coded result headers | `colorizer.rs`, `formatter.rs` | 3.16 |
| 3.19 | `CAPTURE <file>` / `CAPTURE OFF` | `commands/capture.rs` | Phase 1 |
| 3.20 | `LOGIN` command | `commands/login.rs` | Phase 1 |
| 3.21 | `SHOW SESSION <uuid>` | `commands/show.rs` | 2.17 |

**Exit Criteria:** Tab completion matches Python cqlsh behavior. All output formats work. Colored output.

### Phase 4 — COPY & Advanced (Weeks 12-16)

**Goal:** COPY TO/FROM, remaining DESCRIBE, batch mode, full compatibility.

| Task | Description | Deliverable | Depends On |
|------|-------------|-------------|------------|
| 4.1 | COPY TO basic (table -> CSV file) | `commands/copy.rs` | Phase 2 |
| 4.2 | COPY TO all options (DELIMITER, QUOTE, HEADER, NULL, etc.) | `commands/copy.rs` | 4.1 |
| 4.3 | COPY TO token-range splitting (BEGINTOKEN/ENDTOKEN) | `commands/copy.rs` | 4.1 |
| 4.4 | COPY TO concurrent page fetching (MAXREQUESTS) | `commands/copy.rs` | 4.1 |
| 4.5 | COPY TO progress reporting | `commands/copy.rs` | 4.1 |
| 4.6 | COPY FROM basic (CSV file -> table) | `commands/copy.rs` | Phase 2 |
| 4.7 | COPY FROM all options | `commands/copy.rs` | 4.6 |
| 4.8 | COPY FROM parallel workers (NUMPROCESSES) | `commands/copy.rs` | 4.6 |
| 4.9 | COPY FROM rate limiting (INGESTRATE) | `commands/copy.rs` | 4.6 |
| 4.10 | COPY FROM error handling (MAXPARSEERRORS, MAXINSERTERRORS, ERRFILE) | `commands/copy.rs` | 4.6 |
| 4.11 | COPY FROM progress reporting | `commands/copy.rs` | 4.6 |
| 4.12 | DESCRIBE FULL SCHEMA | `commands/describe.rs` | 2.13 |
| 4.13 | DESCRIBE INDEX | `commands/describe.rs` | Phase 2 |
| 4.14 | DESCRIBE MATERIALIZED VIEW | `commands/describe.rs` | Phase 2 |
| 4.15 | DESCRIBE TYPE / TYPES | `commands/describe.rs` | Phase 2 |
| 4.16 | DESCRIBE FUNCTION / FUNCTIONS | `commands/describe.rs` | Phase 2 |
| 4.17 | DESCRIBE AGGREGATE / AGGREGATES | `commands/describe.rs` | Phase 2 |
| 4.18 | `--no-file-io` mode (disable COPY, SOURCE, CAPTURE) | `commands/` | Phase 3 |
| 4.19 | `--tty` flag | `repl.rs` | Phase 1 |
| 4.20 | `--encoding` flag | `config.rs` | Phase 1 |
| 4.21 | `--cqlversion` flag | `session.rs` | Phase 1 |
| 4.22 | `--protocol-version` flag | `session.rs` | Phase 1 |
| 4.23 | Batch/scripting mode exit codes | `runner.rs` | Phase 1 |
| 4.24 | `--debug` flag | `main.rs` | Phase 1 |

**Exit Criteria:** 100% feature parity with Python cqlsh. All flags, commands, and options work.

### Phase 5 — Testing & Benchmarking (Weeks 17-20)

**Goal:** Comprehensive test suite and performance benchmarks.

| Task | Description | Deliverable | Depends On |
|------|-------------|-------------|------------|
| 5.1 | Unit test suite (per-module) | `tests/unit/` | Phase 4 |
| 5.2 | Integration test harness (testcontainers) | `tests/integration/` | Phase 4 |
| 5.3 | Compatibility test suite (run same commands in Python cqlsh and cqlsh-rs, diff output) | `tests/compat/` | Phase 4 |
| 5.4 | Tab completion test suite | `tests/completion/` | Phase 3 |
| 5.5 | COPY TO/FROM test suite (large datasets) | `tests/copy/` | Phase 4 |
| 5.6 | Property-based tests (CQL parsing, formatting) | `tests/proptest/` | Phase 4 |
| 5.7 | Benchmark: startup time | `benches/` | Phase 4 |
| 5.8 | Benchmark: query throughput | `benches/` | Phase 4 |
| 5.9 | Benchmark: COPY TO performance | `benches/` | Phase 4 |
| 5.10 | Benchmark: COPY FROM performance | `benches/` | Phase 4 |
| 5.11 | Benchmark: memory usage profiling | `benches/` | Phase 4 |
| 5.12 | Benchmark: tab completion latency | `benches/` | Phase 3 |
| 5.13 | CI benchmark tracking (criterion + GitHub Actions) | `.github/workflows/` | 5.7-5.12 |
| 5.14 | Cross-platform CI (Linux, macOS, Windows) | `.github/workflows/` | Phase 4 |
| 5.15 | Multi-version Cassandra CI matrix (3.11, 4.0, 4.1, 5.0) | `.github/workflows/` | 5.2 |
| 5.16 | ScyllaDB CI matrix (5.x, 6.x) | `.github/workflows/` | 5.2 |

**Exit Criteria:** >90% test coverage. Benchmarks show improvement over Python cqlsh. CI green on all platforms and DB versions.

### Phase 6 — Polish & Release (Weeks 21-24)

| Task | Description | Deliverable | Depends On |
|------|-------------|-------------|------------|
| 6.1 | Release binary builds (GitHub Releases) | `.github/workflows/release.yml` | Phase 5 |
| 6.2 | Cross-compilation targets | CI config | 6.1 |
| 6.3 | Package manager submissions (brew, apt, cargo install) | Distribution manifests | 6.1 |
| 6.4 | Man page generation | `docs/cqlsh-rs.1` | Phase 4 |
| 6.5 | Migration guide (Python cqlsh -> cqlsh-rs) | `docs/migration.md` | Phase 4 |
| 6.6 | Compatibility divergence documentation | `docs/divergences.md` | Phase 5 |
| 6.7 | Performance comparison report | `docs/benchmarks.md` | Phase 5 |

---

## Sub-Plan Index

Each area below gets its own dedicated research and execution plan document:

| # | Sub-Plan | File | Description |
|---|----------|------|-------------|
| SP1 | **CLI & Configuration** | [`01-cli-and-config.md`](01-cli-and-config.md) | Full CLI flag mapping, cqlshrc parsing, env vars, precedence rules |
| SP2 | **Driver & Connection** | [`02-driver-and-connection.md`](02-driver-and-connection.md) | Driver trait design, scylla integration, connection pooling, auth, SSL/TLS |
| SP3 | **REPL & Line Editing** | [`03-repl-and-editing.md`](03-repl-and-editing.md) | rustyline integration, prompt management, multi-line input, history |
| SP4 | **Statement Parser** | [`04-statement-parser.md`](04-statement-parser.md) | Multi-line buffering, semicolon detection, comment handling, command routing |
| SP5 | **Tab Completion** | [`05-tab-completion.md`](05-tab-completion.md) | Context-aware completion engine, schema cache, keyword lists, file completion |
| SP6 | **Output Formatting** | [`06-output-formatting.md`](06-output-formatting.md) | Table, JSON, CSV, expanded output; type formatting; color; pagination |
| SP7 | **Built-in Commands** | [`07-builtin-commands.md`](07-builtin-commands.md) | All shell commands: DESCRIBE, USE, CONSISTENCY, TRACING, EXPAND, PAGING, etc. |
| SP8 | **COPY TO/FROM** | [`08-copy-to-from.md`](08-copy-to-from.md) | CSV import/export with all options, parallelism, rate limiting, error handling |
| SP9 | **CQL Type System** | [`09-cql-type-system.md`](09-cql-type-system.md) | All CQL types mapping, formatting rules, collection display, UDT handling |
| SP10 | **Testing Strategy** | [`10-testing-strategy.md`](10-testing-strategy.md) | Unit, integration, compatibility, property-based testing; CI configuration |
| SP11 | **Benchmarking** | [`11-benchmarking.md`](11-benchmarking.md) | Performance benchmarks, comparison framework, CI tracking |
| SP12 | **Cross-Platform & Release** | [`12-cross-platform-release.md`](12-cross-platform-release.md) | Multi-platform builds, packaging, distribution |
| SP13 | **Skills Development** | [`13-skills-development.md`](13-skills-development.md) | Team skills assessment, training plans, knowledge prerequisites |

---

## Testing Strategy

### Test Pyramid

```
                    ┌───────────────┐
                    │  Compatibility │  <- Run same commands in Python cqlsh
                    │    Tests       │    and cqlsh-rs, diff output
                    ├───────────────┤
                    │  End-to-End   │  <- Full scenarios: connect -> query ->
                    │   Tests       │    format -> verify output
                ┌───┤              ├───┐
                │   │  Integration │   │  <- Tests against live Cassandra
                │   │    Tests     │   │    (testcontainers-rs)
            ┌───┤   ├─────────────┤   ├───┐
            │   │   │   Property   │   │   │  <- Proptest for parser,
            │   │   │   Tests     │   │   │    formatter, type handling
        ┌───┤   │   ├─────────────┤   │   ├───┐
        │   │   │   │    Unit     │   │   │   │  <- Per-module tests
        │   │   │   │   Tests    │   │   │   │    (pure functions)
        └───┴───┴───┴─────────────┴───┴───┴───┘
```

### Test Infrastructure

| Component | Tool | Purpose |
|-----------|------|---------|
| Unit tests | `cargo test` | Pure function testing, no I/O |
| Integration tests | `testcontainers-rs` | Spin up Cassandra/Scylla in Docker |
| CLI tests | `assert_cmd` + `predicates` | Test binary execution, flags, exit codes |
| Snapshot tests | `insta` | Verify formatted output stability |
| Property tests | `proptest` | Fuzz CQL parsing, type conversions |
| Compatibility tests | Custom harness | Run identical commands in Python cqlsh and cqlsh-rs |
| Coverage | `cargo-tarpaulin` or `llvm-cov` | Track code coverage |
| Mutation testing | `cargo-mutants` | Verify test quality |

### Compatibility Test Approach

```
┌──────────────────┐     ┌──────────────────┐
│   Test Script    │     │   Cassandra      │
│                  │     │   Container      │
│  For each test:  │────>│                  │
│  1. Run cmd in   │     │                  │
│     python cqlsh │     └──────────────────┘
│  2. Run cmd in   │
│     cqlsh-rs     │
│  3. Diff outputs │
│  4. Assert match │
└──────────────────┘
```

Tests cover:
- Every CLI flag combination
- Every shell command with various arguments
- Every CQL type formatting
- Every cqlshrc configuration option
- Tab completion for every context
- Error messages and exit codes
- Edge cases (Unicode, empty results, very long rows, etc.)

---

## Benchmarking Strategy

### Benchmark Suite

| Benchmark | What it Measures | Comparison Target |
|-----------|-----------------|-------------------|
| `startup_cold` | Cold start time (no cache) | Python cqlsh startup |
| `startup_warm` | Warm start time | Python cqlsh startup |
| `connect_auth` | Time to establish authenticated connection | Python cqlsh |
| `query_simple` | Simple SELECT roundtrip | Python cqlsh |
| `query_complex` | Complex JOIN/aggregation query | Python cqlsh |
| `format_table_small` | Format 10-row table output | Python cqlsh |
| `format_table_large` | Format 10,000-row table output | Python cqlsh |
| `format_json` | JSON output formatting | Python cqlsh |
| `format_csv` | CSV output formatting | Python cqlsh |
| `copy_to_small` | Export 1K rows to CSV | Python cqlsh |
| `copy_to_large` | Export 1M rows to CSV | Python cqlsh |
| `copy_from_small` | Import 1K rows from CSV | Python cqlsh |
| `copy_from_large` | Import 1M rows from CSV | Python cqlsh |
| `completion_latency` | Tab completion response time | <50ms target |
| `memory_idle` | Memory usage at idle prompt | Python cqlsh |
| `memory_large_result` | Memory during large result set | Python cqlsh |
| `binary_size` | Release binary size | Tracking only |

### Benchmark Infrastructure

| Component | Tool | Purpose |
|-----------|------|---------|
| Microbenchmarks | `criterion` | Statistical benchmarking with regression detection |
| Macro benchmarks | `hyperfine` | End-to-end timing of full commands |
| Memory profiling | `dhat` / `heaptrack` | Heap allocation tracking |
| CI tracking | `github-action-benchmark` | Track benchmark results over time |
| Comparison | Custom harness | Side-by-side Python cqlsh vs cqlsh-rs |

---

## Required Skills & Team Competencies

### Core Technical Skills

| # | Skill | Level Needed | Used In | Sub-Plans |
|---|-------|-------------|---------|-----------|
| S1 | **Rust (intermediate-advanced)** | Advanced | All modules | All |
| S2 | **Async Rust (Tokio)** | Intermediate | Driver, COPY, session | SP2, SP8 |
| S3 | **CQL protocol & Cassandra internals** | Intermediate | Driver, parser, types | SP2, SP4, SP9 |
| S4 | **Terminal/TUI programming** | Intermediate | REPL, formatter, color | SP3, SP6 |
| S5 | **CLI design patterns** | Intermediate | Config, args, flags | SP1 |
| S6 | **Parser design** | Intermediate | Statement parser, completer | SP4, SP5 |
| S7 | **CSV/data processing** | Intermediate | COPY TO/FROM | SP8 |
| S8 | **SSL/TLS & cryptography** | Basic | Connection, config | SP2 |
| S9 | **Testing methodologies** | Advanced | All test types | SP10 |
| S10 | **Performance profiling** | Intermediate | Benchmarking | SP11 |
| S11 | **CI/CD (GitHub Actions)** | Intermediate | CI, release | SP12 |
| S12 | **Cross-compilation** | Basic | Release builds | SP12 |

### Domain Knowledge

| # | Knowledge Area | Level | Used In |
|---|---------------|-------|---------|
| D1 | **CQL language** (DDL, DML, all statement types) | Deep | Parser, completer, formatter |
| D2 | **Cassandra data model** (keyspaces, tables, types, UDTs) | Deep | DESCRIBE, completer, types |
| D3 | **Cassandra consistency model** | Intermediate | CONSISTENCY command, session |
| D4 | **Cassandra tracing** | Basic | TRACING command |
| D5 | **Python cqlsh internals** | Intermediate | Compatibility testing, edge cases |
| D6 | **ScyllaDB differences from Cassandra** | Basic | Driver, compatibility |

### Crate Expertise

| # | Crate | Skills Needed |
|---|-------|--------------|
| C1 | `scylla` | Connection management, prepared statements, paging, type mapping |
| C2 | `clap` (v4) | Derive API, subcommands, groups, custom validation |
| C3 | `rustyline` | Custom completers, hinters, highlighters, key bindings |
| C4 | `comfy-table` | Column alignment, styles, dynamic width |
| C5 | `tokio` | Runtime setup, channels, spawning, I/O |
| C6 | `serde` + `serde_json` | Serialization for JSON output |
| C7 | `csv` | Reader/writer configuration, custom delimiters |
| C8 | `criterion` | Benchmark groups, custom measurements |
| C9 | `testcontainers` | Container management, wait strategies |
| C10 | `proptest` | Strategy composition, shrinking |
| C11 | `assert_cmd` + `predicates` | CLI testing patterns |
| C12 | `insta` | Snapshot testing workflows |

---

## Skills Development Plan

> Full details in [`13-skills-development.md`](13-skills-development.md)

### Phase 0: Prerequisites (Before Development Starts)

| Week | Focus | Activities | Deliverable |
|------|-------|-----------|-------------|
| Pre-1 | Rust fundamentals refresh | Ownership, lifetimes, traits, error handling | Coding exercises |
| Pre-1 | Async Rust | Tokio tutorial, async/await patterns, pinning | Working async examples |
| Pre-2 | CQL & Cassandra | CQL spec reading, cqlsh usage, data modeling | CQL cheat sheet |
| Pre-2 | Python cqlsh deep-dive | Read Python source, document all behaviors | Behavior catalog |

### Phase 1: Foundation Skills (During Bootstrap)

| Skill | Learning Method | Validation |
|-------|----------------|------------|
| `clap` v4 derive API | Official docs + examples | Build the full CLI parser |
| `scylla` driver basics | Crate docs + examples | Connect and run queries |
| INI parsing with `rust-ini` | Crate docs | Parse sample cqlshrc |
| Error handling with `anyhow`/`thiserror` | Crate docs | Unified error type |

### Phase 2: Interactive Skills (During Usable Shell)

| Skill | Learning Method | Validation |
|-------|----------------|------------|
| `rustyline` advanced | Source code reading, examples | Custom completer + highlighter |
| Terminal manipulation | `crossterm` / ANSI escape codes | Colored output, clearing |
| `comfy-table` formatting | Crate docs | Format all CQL types in tables |
| SSL/TLS with `rustls` | Docs + integration test | TLS connection to Cassandra |

### Phase 3: Advanced Skills (During Completion & COPY)

| Skill | Learning Method | Validation |
|-------|----------------|------------|
| CQL grammar understanding | CQL spec + Python parser source | Context-aware completer |
| Concurrent data processing | Tokio channels, `futures::stream` | Parallel COPY workers |
| CSV edge cases | RFC 4180, Python csv module behavior | Compatibility tests pass |
| Performance profiling | `flamegraph`, `dhat`, `heaptrack` | Identify and fix bottlenecks |

### Phase 4: Quality Skills (During Testing & Benchmarking)

| Skill | Learning Method | Validation |
|-------|----------------|------------|
| `testcontainers-rs` | Crate docs + examples | Integration test suite runs |
| `criterion` benchmarking | Official guide | Benchmark suite with tracking |
| `proptest` | Official book | Property tests for parser |
| `insta` snapshot testing | Crate docs | Output snapshot tests |
| `cargo-tarpaulin` / `llvm-cov` | Tool docs | Coverage reports in CI |

### Skill Development Tracks

Three parallel learning tracks for team members:

```
Track A: Core Engine          Track B: User Interface       Track C: Quality & DevOps
-------------------------     ------------------------      --------------------------
Rust + Async Rust             rustyline deep-dive           Testing frameworks
CQL protocol                  Terminal UI / ANSI            CI/CD setup
scylla driver                 Tab completion design         Benchmarking tools
Statement parsing             Syntax highlighting           Cross-compilation
COPY TO/FROM                  Color schemes                 Release automation
                              Pagination                    Coverage tooling
```

---

## Dependency Candidates (Expanded)

| Crate | Version | Purpose | Alternative | Decision |
|-------|---------|---------|-------------|----------|
| [`scylla`](https://crates.io/crates/scylla) | 0.x | Async Cassandra/Scylla driver | `cdrs-tokio` | Primary: scylla (maintained by ScyllaDB team) |
| [`clap`](https://crates.io/crates/clap) | 4.x | CLI argument parsing | `argh`, `pico-args` | clap (most compatible with complex CLI) |
| [`rustyline`](https://crates.io/crates/rustyline) | 14.x | Line editing, history, completion | `reedline` | rustyline (more mature, better completion API) |
| [`comfy-table`](https://crates.io/crates/comfy-table) | 7.x | Terminal table rendering | `prettytable-rs`, `tabled` | Evaluate all three |
| [`serde`](https://crates.io/crates/serde) | 1.x | Serialization framework | — | Standard choice |
| [`serde_json`](https://crates.io/crates/serde_json) | 1.x | JSON output | — | Standard choice |
| [`tokio`](https://crates.io/crates/tokio) | 1.x | Async runtime | `async-std` | tokio (scylla dependency) |
| [`rust-ini`](https://crates.io/crates/rust-ini) | 0.x | Parse `~/.cqlshrc` | `configparser` | Evaluate both for Python INI compat |
| [`owo-colors`](https://crates.io/crates/owo-colors) | 4.x | Colored terminal output | `colored`, `yansi` | owo-colors (zero-alloc) |
| [`csv`](https://crates.io/crates/csv) | 1.x | CSV reading/writing | — | Standard choice |
| [`anyhow`](https://crates.io/crates/anyhow) | 1.x | Application error handling | `eyre` | anyhow (simpler) |
| [`thiserror`](https://crates.io/crates/thiserror) | 1.x | Library error types | — | Complement to anyhow |
| [`tracing`](https://crates.io/crates/tracing) | 0.1 | Structured logging | `log` | tracing (richer, async-aware) |
| [`tracing-subscriber`](https://crates.io/crates/tracing-subscriber) | 0.3 | Log output formatting | `env_logger` | Pairs with tracing |
| [`chrono`](https://crates.io/crates/chrono) | 0.4 | Timestamp formatting | `time` | chrono (strftime compat with Python) |
| [`uuid`](https://crates.io/crates/uuid) | 1.x | UUID display | — | Standard choice |
| [`num-bigint`](https://crates.io/crates/num-bigint) | 0.4 | Varint display | — | For CQL varint type |
| [`bigdecimal`](https://crates.io/crates/bigdecimal) | 0.4 | Decimal display | `rust_decimal` | For CQL decimal type |
| [`rustls`](https://crates.io/crates/rustls) | 0.23 | TLS implementation | `native-tls` | rustls (pure Rust, no OpenSSL) |
| [`crossterm`](https://crates.io/crates/crossterm) | 0.28 | Terminal manipulation | `termion` | crossterm (cross-platform) |
| [`dirs`](https://crates.io/crates/dirs) | 5.x | Home directory resolution | `home` | dirs (more comprehensive) |

### Test/Bench Dependencies

| Crate | Purpose |
|-------|---------|
| [`criterion`](https://crates.io/crates/criterion) | Statistical benchmarking |
| [`testcontainers`](https://crates.io/crates/testcontainers) | Docker-based integration tests |
| [`assert_cmd`](https://crates.io/crates/assert_cmd) | CLI binary testing |
| [`predicates`](https://crates.io/crates/predicates) | Test assertion helpers |
| [`insta`](https://crates.io/crates/insta) | Snapshot testing |
| [`proptest`](https://crates.io/crates/proptest) | Property-based testing |
| [`tempfile`](https://crates.io/crates/tempfile) | Temporary files for tests |
| [`wiremock`](https://crates.io/crates/wiremock) | Mock servers (if needed) |

---

## Repository Layout (Target)

```
cqlsh-rs/
├── Cargo.toml                    # Workspace root
├── Cargo.lock
├── README.md
├── LICENSE                       # MIT
├── CHANGELOG.md
├── .github/
│   └── workflows/
│       ├── ci.yml                # Lint, test, coverage
│       ├── bench.yml             # Benchmark tracking
│       └── release.yml           # Binary builds & publish
├── docs/
│   ├── plans/
│   │   ├── high-level-design.md  # <- This file
│   │   ├── 01-cli-and-config.md
│   │   ├── 02-driver-and-connection.md
│   │   ├── 03-repl-and-editing.md
│   │   ├── 04-statement-parser.md
│   │   ├── 05-tab-completion.md
│   │   ├── 06-output-formatting.md
│   │   ├── 07-builtin-commands.md
│   │   ├── 08-copy-to-from.md
│   │   ├── 09-cql-type-system.md
│   │   ├── 10-testing-strategy.md
│   │   ├── 11-benchmarking.md
│   │   ├── 12-cross-platform-release.md
│   │   └── 13-skills-development.md
│   ├── migration.md              # Python cqlsh -> cqlsh-rs guide
│   ├── divergences.md            # Documented behavioral differences
│   └── benchmarks.md             # Performance comparison report
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── session.rs
│   ├── repl.rs
│   ├── runner.rs
│   ├── parser.rs
│   ├── formatter.rs
│   ├── colorizer.rs
│   ├── completer.rs
│   ├── types.rs
│   ├── error.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── describe.rs
│   │   ├── copy.rs
│   │   ├── source.rs
│   │   ├── capture.rs
│   │   ├── show.rs
│   │   ├── login.rs
│   │   ├── paging.rs
│   │   ├── expand.rs
│   │   ├── consistency.rs
│   │   ├── tracing_cmd.rs
│   │   ├── help.rs
│   │   └── clear.rs
│   └── driver/
│       ├── mod.rs                # CqlDriver trait
│       └── scylla.rs             # scylla crate implementation
├── tests/
│   ├── unit/                     # Unit tests (also inline in src/)
│   ├── integration/              # Tests against live Cassandra
│   │   ├── common/mod.rs         # Shared test utilities
│   │   ├── test_connect.rs
│   │   ├── test_queries.rs
│   │   ├── test_describe.rs
│   │   ├── test_copy.rs
│   │   └── test_commands.rs
│   ├── compat/                   # Python cqlsh compatibility tests
│   │   ├── harness.rs            # Run command in both, diff output
│   │   ├── test_cli_flags.rs
│   │   ├── test_output_format.rs
│   │   ├── test_type_display.rs
│   │   └── test_completion.rs
│   ├── cli/                      # Binary execution tests (assert_cmd)
│   │   ├── test_flags.rs
│   │   ├── test_execute.rs
│   │   └── test_file.rs
│   └── proptest/                 # Property-based tests
│       ├── test_parser.rs
│       └── test_formatter.rs
├── benches/
│   ├── startup.rs
│   ├── query.rs
│   ├── format.rs
│   ├── copy.rs
│   └── completion.rs
└── fixtures/
    ├── cqlshrc_samples/          # Sample cqlshrc files for testing
    ├── cql_scripts/              # CQL scripts for testing
    └── csv_samples/              # CSV files for COPY testing
```

---

## Risk Register

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| R1 | scylla crate API changes | Medium | Medium | Pin versions, watch releases, maintain abstraction layer |
| R2 | CQL protocol edge cases not covered by scylla crate | Low | High | Contribute upstream or implement raw protocol layer |
| R3 | Python cqlsh undocumented behaviors | High | Medium | Extensive compatibility testing, document divergences |
| R4 | COPY FROM performance parity (Python uses multiprocessing) | Medium | Medium | Use Tokio task parallelism, benchmark early |
| R5 | Tab completion latency on large schemas | Medium | Low | Cache aggressively, async metadata refresh |
| R6 | Cross-platform terminal behavior differences | Medium | Medium | Use crossterm, test on all platforms early |
| R7 | cqlshrc INI parsing edge cases (Python's configparser quirks) | Medium | Low | Test with real-world cqlshrc files from users |
| R8 | Cassandra version-specific behavior differences | Medium | Medium | Multi-version CI matrix |
| R9 | Binary size too large | Low | Low | Feature flags, LTO, strip symbols |
| R10 | Async runtime overhead for simple operations | Low | Low | Profile early, consider sync fallback for simple paths |

---

## Open Questions & Decisions

| # | Question | Options | Status | Decision |
|---|----------|---------|--------|----------|
| Q1 | Driver strategy | a) scylla only, b) scylla + cdrs-tokio, c) pluggable trait | Open | Recommend (a) with (c) trait for future |
| Q2 | Pre-built binaries from day one? | a) Yes (GitHub Releases), b) After v1.0 | Open | Recommend (a) |
| Q3 | COPY FROM in MVP? | a) Phase 2, b) Phase 4 | Open | Recommend (b) — complex feature |
| Q4 | cqlshrc section name compatibility | a) Exact match, b) Superset | Open | Recommend (a) for drop-in replacement |
| Q5 | Color scheme | a) Match Python cqlsh exactly, b) Improved scheme, c) Configurable | Open | Recommend (c) with (a) as default |
| Q6 | Minimum supported Cassandra version | a) 3.0+, b) 3.11+, c) 4.0+ | Open | Recommend (b) |
| Q7 | `reedline` vs `rustyline` | a) rustyline (mature), b) reedline (modern, Nushell) | Open | Needs prototype comparison |
| Q8 | Table rendering crate | a) comfy-table, b) tabled, c) custom | Open | Needs prototype comparison |
| Q9 | How to handle `--browser` flag | a) Implement, b) Accept & ignore, c) Warn & ignore | Open | Recommend (c) |
| Q10 | Plugin hooks (Phase 4) | a) Implement, b) Defer to v2 | Open | Recommend (b) |

---

## Compatibility Target

| Attribute | Target |
|-----------|--------|
| Cassandra versions | 3.11, 4.0, 4.1, 5.0 |
| ScyllaDB versions | 5.x, 6.x |
| Amazon Keyspaces | Best-effort compatibility |
| Astra DB | Best-effort compatibility |
| CQL protocol | v4 (default), v5 (when available) |
| Python cqlsh versions | 6.1.x (Cassandra 4.x bundled), 6.2.x (Cassandra 5.x bundled) |
| Minimum Rust edition | 2021 |
| Minimum Rust toolchain | 1.75+ (stable) |
| Target platforms | linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64, windows-x86_64 |

---

*This document serves as the master plan. Each sub-plan (SP1-SP13) will contain detailed research findings, implementation steps, acceptance criteria, and estimated effort for its specific area.*

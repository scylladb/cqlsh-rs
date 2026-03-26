# Sub-Plan SP3: REPL & Line Editing

> Parent: [high-level-design.md](high-level-design.md) | Phase: 1-2
>
> **Status: COMPLETED** — Core REPL done (2026-03-13). TTY detection and error display improvements deferred to Phase 4.

## Objective

Implement an interactive REPL with line editing, history, multi-line input support, and prompt management that behaves identically to Python cqlsh's interactive mode.

---

## Research Phase

### Tasks

1. **Python cqlsh REPL behavior** — Prompt format, multi-line continuation, history file format
2. **`rustyline` vs `reedline`** — Feature comparison, API for custom completers/highlighters
3. **Key bindings** — Emacs-mode defaults, Ctrl-C/Ctrl-D/Ctrl-L behavior
4. **History behavior** — File location, max size, dedup, multi-line entries
5. **Terminal detection** — TTY vs pipe, `--tty` flag behavior

### Research Deliverables

- [x] Prompt format specification (all states) — `[user@]cqlsh[:keyspace]> `
- [x] Key binding behavior catalog — Emacs mode default via rustyline
- [x] History file format and behavior spec — `~/.cassandra/cql_history`, 1000 entries max
- [x] rustyline vs reedline comparison matrix — Selected rustyline v15 for mature API, custom completer/highlighter support, and better compatibility with Python cqlsh readline behavior

---

## Execution Phase

### Implementation Steps

| Step | Description | Module | Tests | Status |
|------|-------------|--------|-------|--------|
| 1 | Basic REPL loop (stdin readline, no editing) | `repl.rs` | Unit: loop lifecycle | ✅ |
| 2 | rustyline integration with default config | `repl.rs` | Manual: editing works | ✅ |
| 3 | Primary prompt: `cqlsh>` | `repl.rs` | Unit: prompt format | ✅ |
| 4 | Keyspace prompt: `cqlsh:ks>` | `repl.rs` | Unit: keyspace in prompt | ✅ |
| 5 | Continuation prompt: `   ...` for multi-line | `repl.rs` | Unit: multi-line prompt | ✅ |
| 6 | Username@host prompt (when connected) | `repl.rs` | Unit: connected prompt | ✅ |
| 7 | Persistent history (`~/.cassandra/cql_history`) | `repl.rs` | Integration: history persists | ✅ |
| 8 | History max size (configurable) | `repl.rs` | Unit: config integration | ✅ |
| 9 | Ctrl-C handling (cancel current input) | `repl.rs` | Manual: interrupt works | ✅ |
| 10 | Ctrl-D handling (exit on empty line) | `repl.rs` | Manual: exit works | ✅ |
| 11 | Ctrl-L handling (clear screen) | `repl.rs` | Manual: clear works | ✅ (rustyline built-in) |
| 12 | Multi-line input buffering (delegate to parser) | `repl.rs`, `parser.rs` | Unit: semicolon detection | ✅ |
| 13 | Non-interactive mode detection (pipe/redirect) | `repl.rs` | Unit: TTY detection | Pending (SP4) |
| 14 | `--tty` flag override | `repl.rs` | Unit: force TTY mode | Pending (SP4) |
| 15 | Create `ErrorCategory` enum mapping `DbError` variants to Python cqlsh names | `error.rs` | Unit: each variant maps correctly | |
| 16 | Implement `classify_error()` — downcast anyhow chain to extract `DbError` | `error.rs` | Unit: classify each error variant | |
| 17 | Implement `clean_db_error_message()` — strip redundant prefixes | `error.rs` | Unit: cleaning for each error type | |
| 18 | Implement `format_error()` → `"{Category}: {message}"` | `error.rs` | Unit: matches Python cqlsh format | |
| 19 | Integrate in `dispatch_input` — replace `root_cause()` with `classify_error` | `repl.rs` | Integration: verify against known bad queries | |
| 20 | Debug mode: full chain only with `--debug` | `repl.rs` | Unit: flag controls verbosity | |
| 21 | Handle non-query errors (connection refused, timeout, SSL) cleanly | `error.rs` | Unit: connection error classification | |
| 22 | Add `ErrorEnricher` + `HelpProvider` trait stubs for future LLM integration | `error.rs` | Unit: no-op enricher passes through | |

### Error Handling & User-Friendly Error Display

Python cqlsh shows clean, short error messages like `SyntaxException: line 1:7 no viable alternative at input 'invalid_syntax'`. The scylla-rust-driver wraps errors with verbose boilerplate (`Database returned an error: The submitted query has a syntax error, Error message: ...`). We must strip this boilerplate and present errors in the same format as Python cqlsh.

#### Error Categorization

Map scylla `DbError` variants to Python cqlsh error names:

| `DbError` Variant | Python cqlsh Name |
|--------------------|-------------------|
| `SyntaxError` | `SyntaxException` |
| `Invalid` | `InvalidRequest` |
| `Unauthorized` | `Unauthorized` |
| `Unavailable` | `Unavailable` |
| `ReadTimeout` | `ReadTimeout` |
| `WriteTimeout` | `WriteTimeout` |
| `ConfigError` | `ConfigurationException` |
| `AlreadyExists` | `AlreadyExists` |
| `Overloaded` | `Overloaded` |
| `IsBootstrapping` | `IsBootstrapping` |
| `TruncateError` | `TruncateError` |
| `ReadFailure` | `ReadFailure` |
| `WriteFailure` | `WriteFailure` |
| `FunctionFailure` | `FunctionFailure` |
| Other/Unknown | `ServerError` |

#### Error Message Cleaning

Strip redundant prefixes added by the driver:
- Remove `"Database returned an error: "` wrapper
- Remove `"The submitted query has a syntax error, "` prefix
- Remove `"Error message: "` prefix
- Show just `{Category}: {cleaned_message}`

#### Future: LLM-Powered Error Assistance

> **Status**: Vision / Not yet planned

An opt-in AI-powered error assistance layer for future implementation:

- **Configuration**: `[ai]` section in `cqlshrc` with `provider`, `model`, `api_key_env` keys
- **CLI flag**: `--ai-help` (disabled by default)
- **Capabilities**:
  - Suggest fixes for common errors (typos, missing keyspace, wrong types)
  - Schema-aware suggestions ("Column 'namee' not found. Did you mean 'name'?")
  - Enhanced `HELP <topic>` backed by LLM context
- **Privacy**: Never send table data to LLM — only schema metadata and error text
- **Local-first**: Prioritize Ollama and local models over cloud APIs
- **Graceful degradation**: If LLM is unavailable, fall back to standard error display silently

### Acceptance Criteria

- [x] Prompt matches Python cqlsh format in all states
- [x] Line editing (arrow keys, Home/End, word movement) works
- [x] History persists across sessions in `~/.cassandra/cql_history`
- [x] Multi-line input shows continuation prompt
- [x] Ctrl-C cancels input without exiting
- [x] Ctrl-D exits on empty line
- [ ] Pipe/redirect mode works without editing features (Phase 4)
- [ ] `--tty` forces interactive mode even in pipe (Phase 4)
- [x] Syntax errors display as `SyntaxException: <msg>` (no "Database returned an error" wrapper)
- [x] Invalid queries display as `InvalidRequest: <msg>`
- [x] No backtraces in normal mode; `--debug` shows full error chain
- [x] Connection errors show clean one-liner messages

---

## Skills Required

- `rustyline` API (C3)
- Terminal programming (S4)
- Signal handling in Rust (S1)

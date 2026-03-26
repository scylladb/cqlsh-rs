# cqlsh-rs: Plan Review & Remaining Tasks

> **Generated:** 2026-03-26
> **Based on:** `docs/progress.json`, all SP01-SP19 plan documents, codebase analysis
> **Overall Progress:** 61/108 tasks completed (56%), Phases 1-3 complete, Phase 5 in progress

---

## Plan Status Summary

| Plan | Title | Status | Notes |
|------|-------|--------|-------|
| SP1 | CLI & Configuration | COMPLETED | 91 tests, all flags implemented |
| SP2 | Driver & Connection | COMPLETED | SSL/TLS, auth, pooling, metadata |
| SP3 | REPL & Line Editing | COMPLETED | rustyline, history, multi-line, prompts |
| SP4 | Statement Parser | COMPLETED | O(n) incremental parser, 50+ tests |
| SP5 | Tab Completion | COMPLETED | Context-aware, schema-aware, file paths |
| SP6 | Output Formatting | COMPLETED | Tabular, expanded, color, type-aware |
| SP7 | Built-in Commands | PARTIALLY COMPLETE | 19/25+ commands done, DESCRIBE extensions remain |
| SP8 | COPY TO/FROM | PARTIALLY COMPLETE | COPY TO done, COPY FROM not started |
| SP9 | CQL Type System | COMPLETED | All 25 types, collections, UDTs |
| SP10 | Testing Strategy | IN PROGRESS | 327 unit tests, 0 integration tests |
| SP11 | Benchmarking | IN PROGRESS | Startup benchmarks done, CI dashboard live |
| SP12 | Cross-Platform & Release | NOT STARTED | Phase 6 |
| SP13 | Skills Development | REFERENCE | Living document, not a deliverable |
| SP14 | Documentation & LLM Skills | NOT STARTED | Phase 6 |
| SP15 | AI CI Failure Summaries | NOT STARTED | Phase 5 |
| SP16 | Upstream PR Review | COMPLETED | PR #150, #151 fixes incorporated |
| SP17 | AI Assistant (`--ai-help`) | NOT STARTED | Value-add feature, post-v1 |
| SP18 | Unified CQL Lexer | NOT STARTED | Cross-cutting refactor, post-v1 |
| SP19 | Test Gap Analysis | IN PROGRESS | Gap analysis done, execution pending |

---

## Phase Completion Detail

### Phase 1: Bootstrap MVP — COMPLETED (2026-03-01 to 2026-03-15)
All 15/15 tasks done. CLI parsing, config, driver, basic REPL.

### Phase 2: Usable Shell — COMPLETED (2026-03-15 to 2026-03-18)
All 22/22 tasks done. Formatting, commands, type system, error handling.

### Phase 3: Tab Completion & Output — COMPLETED (2026-03-18 to 2026-03-22)
All 21/21 tasks done. Completion engine, colorizer, schema cache.

### Phase 4: COPY & Advanced — NOT STARTED (0/24 tasks)
COPY FROM, remaining DESCRIBE variants, LOGIN, non-interactive improvements.

### Phase 5: Testing & Benchmarks — IN PROGRESS (3/16 tasks)
Benchmark infra done. Integration tests and coverage tooling remain.

### Phase 6: Polish & Release — NOT STARTED (0/10 tasks)
Cross-platform builds, packaging, documentation site.

---

## Remaining Tasks by Phase

### Phase 4: COPY & Advanced (24 tasks)

#### 4.1 Non-Interactive & Shell Improvements
- [ ] Non-interactive mode detection (pipe/redirect stdin)
- [ ] `--tty` flag override for forced interactive mode
- [ ] DEBUG command toggle
- [ ] UNICODE command (character handling info)
- [ ] LOGIN command (re-authenticate without reconnect)
- [ ] Exit code consistency (0 success, 1 CQL error, 2 connection error)

#### 4.2 DESCRIBE Extensions
- [ ] DESCRIBE FULL SCHEMA
- [ ] DESCRIBE INDEX `<name>`
- [ ] DESCRIBE MATERIALIZED VIEW `<name>`
- [ ] DESCRIBE TYPE / TYPES
- [ ] DESCRIBE FUNCTION / FUNCTIONS
- [ ] DESCRIBE AGGREGATE / AGGREGATES
- [ ] SHOW SESSION `<uuid>` (tracing session detail)

#### 4.3 COPY FROM Implementation
- [ ] CSV parsing with `csv` crate
- [ ] All 19 COPY FROM options (CHUNKSIZE, TTL, NUMPROCESSES, etc.)
- [ ] Prepared statement batching
- [ ] Token bucket rate limiting (INGESTRATE)
- [ ] Parallel worker tasks (Tokio async)
- [ ] Error file logging (ERRFILE)
- [ ] MAXPARSEERRORS / MAXINSERTERRORS thresholds
- [ ] Stdin input support
- [ ] Progress reporting (REPORTFREQUENCY)

#### 4.4 Miscellaneous
- [ ] `--safe-mode` flag with DROP/TRUNCATE confirmation prompts (SP16 PR #147)

### Phase 5: Testing & Benchmarks (13 remaining of 16)

- [ ] Integration test infrastructure (testcontainers-rs with Cassandra)
- [ ] Integration tests: connection lifecycle
- [ ] Integration tests: CRUD operations
- [ ] Integration tests: DDL operations
- [ ] Integration tests: DESCRIBE commands against live DB
- [ ] Integration tests: COPY TO against live DB
- [ ] Integration tests: COPY FROM against live DB
- [ ] Integration tests: SSL/TLS connection
- [ ] Snapshot tests (insta) for output formatting
- [ ] Property-based tests (proptest) for parser and type roundtrips
- [ ] Compatibility tests: output comparison vs Python cqlsh
- [ ] CI matrix: OS (ubuntu, macos, windows) x Rust (stable, beta)
- [ ] CI matrix: DB versions (Cassandra 3.11/4.0/4.1/5.0, ScyllaDB 5.4/6.0)
- [ ] Additional benchmarks: parser, formatter, completion
- [ ] AI CI failure summaries (SP15)
- [ ] Code coverage tooling (cargo-tarpaulin, >90% target)

### Phase 6: Polish & Release (10 tasks)

- [ ] Cross-compilation setup (cross-rs for Linux aarch64)
- [ ] Release workflow: tag → build → GitHub Release
- [ ] Binary size optimization (<10MB standard)
- [ ] crates.io publication
- [ ] Homebrew formula
- [ ] Docker image
- [ ] Man page generation (clap_mangen)
- [ ] Shell completion packaging (bash/zsh/fish in release artifacts)
- [ ] Documentation site (mdBook) with migration guide
- [ ] LLM-oriented docs (llms.txt, AGENTS.md)

---

## Execution Prompts

Below are copy-paste-ready prompts for Claude Code to execute each remaining task group. Each prompt is self-contained and references the relevant plan documents.

---

### Prompt 1: Non-Interactive Mode & Shell Improvements (Phase 4.1)

```
Using the /development-process skill, implement Phase 4.1 from docs/plans/phase4-copy-and-advanced.md:

1. Add pipe/redirect stdin detection — when stdin is not a TTY, skip the REPL and read statements line-by-line (like Python cqlsh behavior)
2. Implement the `--tty` flag override to force interactive mode even when stdin is not a TTY
3. Implement the DEBUG command that toggles debug output (prints internal state, driver debug info)
4. Implement the UNICODE command that displays character encoding info
5. Implement the LOGIN command that re-authenticates with new credentials without disconnecting
6. Standardize exit codes: 0=success, 1=CQL error, 2=connection error

Reference: docs/plans/07-builtin-commands.md for command specs, docs/plans/phase4-manual-test-plan.md for test cases. Write unit tests for each command. Update docs/progress.json when done.
```

---

### Prompt 2: DESCRIBE Extensions (Phase 4.2)

```
Using the /development-process skill, implement the remaining DESCRIBE sub-commands from docs/plans/07-builtin-commands.md:

1. DESCRIBE FULL SCHEMA — include system keyspaces in schema dump
2. DESCRIBE INDEX <name> — reconstruct CREATE INDEX DDL from system_schema.indexes
3. DESCRIBE MATERIALIZED VIEW <name> — reconstruct CREATE MATERIALIZED VIEW DDL from system_schema.views
4. DESCRIBE TYPE[S] [name] — list or show CREATE TYPE DDL for UDTs
5. DESCRIBE FUNCTION[S] [name] — list or show CREATE FUNCTION DDL for UDFs
6. DESCRIBE AGGREGATE[S] [name] — list or show CREATE AGGREGATE DDL
7. SHOW SESSION <uuid> — fetch and display tracing session details by UUID

All DDL reconstruction must query system_schema tables. Match Python cqlsh output format exactly. Add unit tests and update tab completion for new sub-commands. Reference: docs/plans/phase4-manual-test-plan.md for expected outputs. Update docs/progress.json when done.
```

---

### Prompt 3: COPY FROM Implementation (Phase 4.3-4.4)

```
Using the /development-process skill, implement COPY FROM as specified in docs/plans/08-copy-to-from.md:

Phase 1 — Core COPY FROM:
1. Parse COPY FROM command syntax: COPY ks.table [(col1, col2)] FROM 'file' WITH opt=val AND ...
2. CSV parsing using the `csv` crate with configurable DELIMITER, QUOTE, ESCAPE, HEADER, NULL, ENCODING
3. Type conversion from CSV strings to CqlValue for all 25 CQL types
4. INSERT execution using prepared statements
5. Row count reporting and timing

Phase 2 — Advanced Options:
6. TTL option (set TTL on inserted rows)
7. CHUNKSIZE / MAXBATCHSIZE / MINBATCHSIZE for batching control
8. PREPAREDSTATEMENTS toggle (prepared vs simple)
9. NUMPROCESSES — parallel Tokio tasks for concurrent inserts
10. INGESTRATE — token bucket rate limiting
11. MAXATTEMPTS — retry failed inserts
12. MAXPARSEERRORS / MAXINSERTERRORS — error thresholds before abort
13. ERRFILE — log failed rows to error file
14. REPORTFREQUENCY — periodic progress reporting
15. Stdin input support (COPY FROM STDIN)

Write unit tests for option parsing and type conversion. Write integration test stubs for containerized testing. Reference: docs/plans/phase4-copy-test-plan.md for test scenarios. Update docs/progress.json when done.
```

---

### Prompt 4: Safe Mode (Phase 4 — SP16)

```
Implement --safe-mode as described in docs/plans/16-upstream-pr-review.md (PR #147):

1. Add `--safe-mode` CLI flag to CliArgs in src/cli.rs
2. Add `safe_mode` config option in [ui] section of cqlshrc
3. Before executing DROP or TRUNCATE statements, prompt "Are you sure? [y/N]" on stderr
4. In non-interactive mode (-e, -f), --safe-mode should abort on DROP/TRUNCATE with an error message
5. Add unit tests for safe mode flag parsing and prompt logic

Update docs/progress.json when done.
```

---

### Prompt 5: Integration Test Infrastructure (Phase 5)

```
Using the /rust-testing skill, set up integration test infrastructure:

1. Add testcontainers-rs dependency to Cargo.toml [dev-dependencies]
2. Create tests/integration/mod.rs with shared test fixtures:
   - Cassandra container setup/teardown
   - Helper to create test keyspace and tables
   - Helper to get a connected CqlshSession for testing
3. Write initial integration tests:
   - Connection lifecycle (connect, query, disconnect)
   - CREATE/DROP KEYSPACE
   - CREATE/DROP TABLE
   - INSERT/SELECT/UPDATE/DELETE roundtrip
   - USE keyspace switches prompt
   - DESCRIBE commands against live schema
4. Configure CI to run integration tests with Docker (Cassandra 4.1)
5. Add #[ignore] attribute so `cargo test` skips integration tests by default; run with `cargo test -- --ignored`

Reference: docs/plans/10-testing-strategy.md and docs/plans/19-test-gap-analysis.md. Update docs/progress.json when done.
```

---

### Prompt 6: Snapshot & Property-Based Tests (Phase 5)

```
Using the /rust-testing skill, add snapshot and property-based tests:

Snapshot tests (insta crate):
1. Add insta to dev-dependencies
2. Snapshot tests for tabular output (5, 10, 50 rows with various types)
3. Snapshot tests for expanded output mode
4. Snapshot tests for JSON output format
5. Snapshot tests for DESCRIBE output (keyspace, table, schema)
6. Snapshot tests for HELP command output
7. Snapshot tests for error message formatting

Property-based tests (proptest crate):
8. Add proptest to dev-dependencies
9. Parser roundtrip: arbitrary CQL statements parse without panic
10. Type format/parse idempotence: format(parse(s)) == s for valid values
11. Config merge associativity: merging configs is deterministic
12. CSV roundtrip: write then read produces same data

Reference: docs/plans/10-testing-strategy.md. Update docs/progress.json when done.
```

---

### Prompt 7: CI Matrix & Coverage (Phase 5)

```
Using the /github-actions skill, expand the CI pipeline:

1. Add OS matrix: ubuntu-latest, macos-latest, windows-latest
2. Add Rust toolchain matrix: stable, beta
3. Add integration test job with Cassandra service container (4.1)
4. Add ScyllaDB service container job (6.0)
5. Add code coverage job using cargo-tarpaulin, upload to codecov.io
6. Add benchmark regression detection on PRs (compare against main)
7. Ensure the progress.yml workflow still works

Reference: docs/plans/10-testing-strategy.md for CI matrix spec, docs/plans/11-benchmarking.md for benchmark CI. Update docs/progress.json when done.
```

---

### Prompt 8: Additional Benchmarks (Phase 5)

```
Using the /rust-performance skill, add micro-benchmarks per docs/plans/11-benchmarking.md:

Parser benchmarks (benches/parser.rs):
1. parse_simple_select — "SELECT * FROM ks.table WHERE id = 1;"
2. parse_multiline — 10-line INSERT with collections
3. parse_batch — 100 statements separated by semicolons
4. parse_large — 1MB statement (regression test for O(n²))

Formatter benchmarks (benches/formatter.rs):
5. format_table_10 — 10 rows, 5 columns
6. format_table_100 — 100 rows, 10 columns
7. format_table_1000 — 1000 rows, 10 columns
8. format_json_100 — 100 rows JSON output
9. format_csv_100 — 100 rows CSV output
10. format_each_type — one row per CQL type

Completion benchmarks (benches/completion.rs):
11. complete_keyword — "SEL" → "SELECT"
12. complete_table — "SELECT * FROM " with 100 tables in cache
13. complete_column — "SELECT " with table context, 50 columns

Add all to CI benchmark tracking. Update docs/progress.json when done.
```

---

### Prompt 9: AI CI Failure Summaries (Phase 5 — SP15)

```
Using the /ci-failure-analysis skill, implement SP15 from docs/plans/15-ai-ci-failure-summaries.md:

1. Add cargo-nextest for structured JUnit XML test output
2. Create .github/workflows/ci-failure-analysis.yml triggered by workflow_run
3. On CI failure: collect JUnit XML + raw logs
4. Invoke claude-code-action to classify failures (compilation, test, lint, infra, dependency, config, unknown)
5. Post collapsed PR comment with per-job failure analysis, root cause, and suggested fix
6. Add flaky test detection (same test fails intermittently across runs)
7. Add re-run link in comment footer

Reference: docs/plans/15-ai-ci-failure-summaries.md for full spec. Update docs/progress.json when done.
```

---

### Prompt 10: Cross-Platform Release Pipeline (Phase 6)

```
Using the /github-actions skill, implement the release pipeline from docs/plans/12-cross-platform-release.md:

1. Create .github/workflows/release.yml triggered by tag push (v*)
2. Build matrix: Linux x86_64 (musl static), Linux aarch64 (cross-rs), macOS x86_64, macOS aarch64, Windows x86_64
3. Run integration tests on each platform
4. Generate SHA256 checksums for all binaries
5. Create GitHub Release with all binaries and checksums
6. Publish to crates.io (cargo publish)
7. Generate man pages with clap_mangen, include in release
8. Generate shell completions (bash/zsh/fish), include in release
9. Build and push Docker image (alpine-based, static binary)
10. Create Homebrew formula (tap or core)

Reference: docs/plans/12-cross-platform-release.md for targets and size budgets. Update docs/progress.json when done.
```

---

### Prompt 11: Documentation Site (Phase 6)

```
Implement the documentation site from docs/plans/14-documentation.md:

1. Initialize mdBook project in docs/book/
2. Write chapters:
   - Getting Started (install, first connection)
   - Installation (all platforms: cargo, brew, docker, binary)
   - Configuration Reference (all cqlshrc sections with examples)
   - Command Reference (all 25 shell commands)
   - CLI Reference (all flags with examples)
   - COPY Guide (COPY TO and COPY FROM with examples)
   - Migration Guide (Python cqlsh → cqlsh-rs, known divergences)
   - Troubleshooting
3. Create .github/workflows/docs.yml to build and deploy to GitHub Pages
4. Add link checking (lychee-action)
5. Create llms.txt and llms-full.txt for LLM consumption
6. Serve docs at GitHub Pages URL

Reference: docs/plans/14-documentation.md. Update docs/progress.json when done.
```

---

### Prompt 12: Unified CQL Lexer Refactor (Post-v1 — SP18)

```
Implement the unified CQL lexer from docs/plans/18-cql-lexer.md:

1. Create src/cql_lexer.rs with a hand-written state machine tokenizer
2. Token types: Keyword, Identifier, QuotedIdentifier, StringLiteral, NumberLiteral, BlobLiteral, Operator, Punctuation, Whitespace, LineComment, BlockComment, Unknown
3. Add grammar context tracking (ExpectTable, ExpectColumn, ExpectType, etc.)
4. Migrate src/colorizer.rs to use cql_lexer (fix false keyword highlights on identifiers)
5. Migrate src/completer.rs to use cql_lexer (improve context detection and ordering)
6. Migrate src/parser.rs to use cql_lexer (reuse tokenizer, remove duplicate logic)
7. Delete all redundant tokenization code
8. Verify all existing tests still pass
9. Add lexer-specific unit tests (100+ covering all token types and edge cases)

Reference: docs/plans/18-cql-lexer.md. This is a cross-cutting refactor — run full test suite after each migration step.
```

---

### Prompt 13: AI Assistant Feature (Post-v1 — SP17)

```
Implement the --ai-help feature from docs/plans/17-ai-assistant-help.md:

Phase 1 — Infrastructure:
1. Add `ai-help` Cargo feature flag (disabled by default)
2. Add llama-cpp-2 dependency behind feature gate
3. Implement model download manager (Qwen2.5-Coder-0.5B-Instruct Q4_K_M GGUF, ~350MB)
4. Cache to platform-specific dir (~/.cache/cqlsh-rs/models/ on Linux)
5. Add [ai] config section and CQLSH_AI_HELP env var

Phase 2 — Inference:
6. Load model lazily on first error
7. System prompt: "Terse CQL diagnostic assistant, max 2 sentences"
8. User prompt: "CQL Statement: {stmt}\nError: {error}\nSuggest a fix:"
9. Generation: max_tokens=128, temperature=0.1, repeat_penalty=1.1
10. Hard timeout: 15 seconds

Phase 3 — CLI Integration:
11. --ai-help flag to enable
12. --ai-clear-cache to remove downloaded model
13. --ai-threads <N> for inference thread count
14. Display AI suggestion below error message with "[AI suggestion]" prefix
15. Graceful degradation: any failure falls back silently to standard error display

Phase 4 — Testing:
16. Unit tests for config, download manager, prompt formatting
17. Integration test with mock model
18. Cross-platform testing (Linux, macOS, Windows)

Reference: docs/plans/17-ai-assistant-help.md. Ensure zero impact on binary size and startup when feature is disabled.
```

---

## Recommended Execution Order

| Order | Prompt | Phase | Priority | Dependencies |
|-------|--------|-------|----------|-------------|
| 1 | Prompt 2: DESCRIBE Extensions | 4.2 | High | None |
| 2 | Prompt 1: Non-Interactive & Shell | 4.1 | High | None |
| 3 | Prompt 4: Safe Mode | 4 | Medium | None |
| 4 | Prompt 3: COPY FROM | 4.3-4.4 | High | None (COPY TO already done) |
| 5 | Prompt 5: Integration Test Infra | 5 | High | Phase 4 features to test |
| 6 | Prompt 6: Snapshot & Property Tests | 5 | Medium | None |
| 7 | Prompt 8: Additional Benchmarks | 5 | Medium | None |
| 8 | Prompt 7: CI Matrix & Coverage | 5 | Medium | Integration tests exist |
| 9 | Prompt 9: AI CI Failure Summaries | 5 | Low | CI pipeline exists |
| 10 | Prompt 10: Release Pipeline | 6 | High | All features done |
| 11 | Prompt 11: Documentation Site | 6 | High | All features done |
| 12 | Prompt 12: Unified Lexer | Post-v1 | Low | All tests passing |
| 13 | Prompt 13: AI Assistant | Post-v1 | Low | v1 released |

---

## Key Risks for Remaining Work

1. **COPY FROM complexity** — 19 options, parallel workers, rate limiting. This is the largest single task remaining. Budget 2-3 sessions.
2. **Integration test flakiness** — testcontainers-rs + Cassandra can be slow and flaky in CI. May need retry logic and generous timeouts.
3. **Cross-platform builds** — Windows and aarch64 Linux may surface platform-specific issues. Test early.
4. **Output compatibility** — Character-for-character match with Python cqlsh is hard. Snapshot tests will catch regressions but initial alignment requires manual comparison.

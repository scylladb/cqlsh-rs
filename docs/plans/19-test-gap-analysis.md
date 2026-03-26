# Sub-Plan SP19: Test Gap Analysis — Python cqlsh vs cqlsh-rs

> Parent: [high-level-design.md](high-level-design.md) | Cross-cutting
>
> **This is a living document.** Update it as development progresses.
> **Status: IN PROGRESS** — Gap analysis complete (327 tests catalogued, critical gaps identified). Execution of gap-closing tests pending Phase 5.

## Objective

Identify test coverage gaps between cqlsh-rs and the Python cqlsh test suites (scylla-cqlsh unit tests + scylla-dtest integration tests), and create an actionable plan to close them.

---

## Source Test Suites

### 1. scylla-cqlsh unit tests (`pylib/cqlshlib/test/`)

| Test File | Tests | Category | cqlsh-rs Coverage |
|-----------|-------|----------|-------------------|
| `test_cql_parsing.py` | 23 | CQL parsing (SELECT, INSERT, UPDATE, DELETE, CREATE, DROP, USE, etc.) | **PARTIAL** — parser.rs has 50+ tests but no CQL-statement-type-specific parsing tests |
| `test_cqlsh_completion.py` | 20+ | Tab completion (SELECT, INSERT, UPDATE, DELETE, BATCH, CREATE, DROP, DESCRIBE, ALTER, GRANT, REVOKE) | **PARTIAL** — completer.rs has 17 tests but lacks per-statement-type completion tests |
| `test_cqlsh_output.py` | 25+ | Output formatting (color, no-color, numeric, timestamp, boolean, null, string, blob, prompt, DESCRIBE output, SHOW, HELP, multiline, errors) | **PARTIAL** — formatter.rs has 9 tests, colorizer.rs has 18; missing output comparison tests |
| `test_copyutil.py` | 11 | COPY internals (ranges, export to stdout, import validation, error handling, retry) | **MINIMAL** — copy.rs has 29 parsing/formatting tests but no integration tests |
| `test_escape_decoding.py` | 13 | Escape sequence decoding (hex, standard, backslash, quote) | **NONE** |
| `test_escape_roundtrip.py` | 7 | Escape sequence round-trip (control chars, backslash, mixed) | **NONE** |
| `test_escape_sequences.py` | 5 | Escape sequences in queries (hex, standard, backslash) | **NONE** |
| `test_formatting.py` | 2 | Formatting edge cases (multiple semicolons, spaces in DESCRIBE) | **NONE** |
| `test_unicode.py` | 5 | Unicode handling (values, identifiers, multiline, DESCRIBE, escape) | **NONE** |
| `test_authproviderhandling.py` | 16 | Auth provider config (cqlshrc, CLI args, legacy) | **NONE** — no auth provider tests |
| `test_compression.py` | 5 | Compression settings (default, cqlshrc, CLI flag) | **NONE** — no compression support |
| `test_constants.py` | 1 | CQL reserved keywords list | **PARTIAL** — keyword list exists in colorizer.rs but untested against spec |
| `test_legacy_auth.py` | 1 | Legacy auth credentials | **NONE** |
| `test_vector_formatting.py` | 9 | Vector type parsing and formatting | **NONE** — vector type not yet supported |

### 2. scylla-dtest integration tests (`cqlsh_tests/`)

| Test File | Tests | Category | cqlsh-rs Coverage |
|-----------|-------|----------|-------------------|
| **cqlsh_tests.py — TestCqlsh** | | | |
| `test_simple_insert` | 1 | Basic INSERT + SELECT | **NONE** (integration) |
| `test_past_and_future_dates` | 1 | Date/timestamp handling | **NONE** |
| `test_eat_glass` | 1 | Unicode (UTF-8 glass-eating test) | **NONE** |
| `test_source_glass` | 1 | SOURCE with Unicode | **NONE** |
| `test_with_empty_values` | 1 | Empty string values | **NONE** |
| `test_tracing_from_system_traces` | 1 | TRACING + system_traces query | **NONE** |
| `test_select_element_inside_udt` | 1 | UDT field access in SELECT | **NONE** |
| `test_list_queries` | 1 | Various list operations | **NONE** |
| `test_describe` | 1 | Comprehensive DESCRIBE tests | **PARTIAL** — unit tests for DDL output |
| `test_describe_describes_non_default_compaction_parameters` | 1 | DESCRIBE with compaction | **NONE** |
| `test_describe_on_non_reserved_keywords` | 1 | DESCRIBE with keyword names | **NONE** |
| `test_describe_mv` | 1 | DESCRIBE MATERIALIZED VIEW | **NONE** (integration) |
| `test_copy_to` | 1 | COPY TO basic | **NONE** (integration) |
| `test_float_formatting` | 1 | Float display precision | **PARTIAL** — unit tests exist |
| `test_int_values` | 1 | Integer type display | **PARTIAL** — unit tests exist |
| `test_datetime_values` | 1 | Timestamp formatting | **PARTIAL** — unit tests exist |
| `test_tracing` | 1 | TRACING ON/OFF | **NONE** (integration) |
| `test_connect_timeout` | 1 | --connect-timeout flag | **NONE** |
| `test_describe_round_trip` | 1 | DESCRIBE → CREATE round trip | **NONE** |
| `test_materialized_view` | 1 | MV operations | **NONE** |
| `test_clear` / `test_cls` | 2 | CLEAR/CLS command | **NONE** (integration) |
| `test_batch` | 1 | BATCH statement | **NONE** |
| **cqlsh_tests.py — TestCqlshSmoke** | | | |
| `test_uuid` | 1 | UUID operations | **NONE** |
| `test_commented_lines` | 1 | Comments in input | **PARTIAL** — parser tests |
| `test_colons_in_string_literals` | 1 | Colons in strings | **PARTIAL** — parser tests |
| `test_select/insert/update/delete` | 4 | Basic CRUD operations | **NONE** (integration) |
| `test_batch` | 1 | BATCH statement | **NONE** |
| `test_create/drop_keyspace` | 2 | DDL operations | **NONE** (integration) |
| `test_create/drop_table` | 2 | DDL operations | **NONE** |
| `test_truncate` | 2 | TRUNCATE (with/without LIMIT) | **NONE** |
| `test_alter_table` | 1 | ALTER TABLE | **NONE** |
| `test_use_keyspace` | 1 | USE command | **PARTIAL** — unit tests |
| `test_create/drop_index` | 2 | INDEX operations | **NONE** |
| **cqlsh_tests.py — TestCqlLogin** | | | |
| `test_login_keeps_keyspace` | 1 | LOGIN preserves keyspace | **NONE** |
| `test_login_rejects_bad_pass` | 1 | LOGIN with wrong password | **NONE** |
| `test_login_authenticates_correct_user` | 1 | LOGIN with correct credentials | **NONE** |
| `test_login_allows_bad_pass_and_continued_use` | 1 | LOGIN failure doesn't disconnect | **NONE** |
| **cqlsh_copy_tests.py — TestCqlshCopy** | | | |
| `test_list_data` | 1 | COPY with list columns | **NONE** |
| `test_tuple_data` | 1 | COPY with tuple columns | **NONE** |
| `test_colon/letter/number_delimiter` | 3 | Custom delimiters | **NONE** (integration) |
| `test_null_as_null_indicator` | 2 | NULL indicators | **NONE** |
| `test_writing/reading_use_header` | 2 | HEADER option | **NONE** (integration) |
| `test_reading_counter` | 3 | Counter columns | **NONE** |
| `test_writing_with_timeformat` | 1 | DATETIMEFORMAT | **NONE** |
| `test_reading_with_ttl` | 1 | TTL on import | **NONE** |
| `test_explicit_column_order_writing/reading` | 2 | Column ordering | **NONE** |
| `test_quoted_column_names_*` | 4 | Quoted identifiers | **NONE** |
| `test_read_valid/invalid_*` | 4 | Data validation | **NONE** |
| `test_all_datatypes_write/read/round_trip` | 3 | All CQL types | **NONE** |
| `test_wrong_number_of_columns` | 1 | Column count mismatch | **NONE** |
| `test_round_trip_murmur3` | 1 | Token-range round trip | **NONE** |
| `test_source_copy_round_trip` | 1 | SOURCE + COPY | **NONE** |
| `test_bulk_round_trip_*` | 3 | Large dataset COPY | **NONE** |
| `test_copy_to/from_with_*_failures` | 4 | Error handling | **NONE** |
| `test_copy_to/from_with_child_process_crashing` | 2 | Crash recovery | **NONE** |

---

## Gap Summary

### cqlsh-rs Current Coverage (327 tests)

| Module | Unit Tests | Integration Tests |
|--------|-----------|-------------------|
| CLI argument parsing | 34 | 19 |
| Config file parsing | 35+ | 0 |
| CQL statement parser | 50+ | 0 |
| Tab completion | 17 | 0 |
| Output formatting | 9 | 0 |
| Colorizer | 18 | 0 |
| Error classification | 11 | 0 |
| DESCRIBE | 11 | 0 |
| COPY TO/FROM | 29 | 0 |
| Driver types | 14 | 0 |
| Driver conversion | 16 | 0 |
| Schema cache | 30 | 0 |
| Session/USE | 8 | 0 |
| REPL | 25 | 0 |
| Shell completions | 5 | 0 |

### Critical Gaps (Priority Order)

#### P0 — No integration tests at all
cqlsh-rs has **zero** integration tests that connect to a real database. All 327 tests are unit tests or CLI binary tests. Python cqlsh has 80+ integration tests.

#### P1 — Missing test categories entirely

| Category | Python Tests | cqlsh-rs Tests | Gap |
|----------|-------------|---------------|-----|
| **Escape sequences** | 25 tests (3 files) | 0 | Full gap |
| **Unicode handling** | 5 tests | 0 | Full gap |
| **Auth provider** | 17 tests | 0 | Full gap |
| **Output comparison** | 25+ tests | 0 | Full gap — no tests verify output matches Python cqlsh |
| **COPY integration** | 35+ tests | 0 | Full gap — only parsing unit tests |
| **LOGIN** | 4 tests | 0 | Full gap |
| **DESCRIBE integration** | 5+ tests | 0 | Only DDL-generation unit tests |

#### P2 — Partial coverage, needs expansion

| Category | Python Tests | cqlsh-rs Tests | Gap |
|----------|-------------|---------------|-----|
| **CQL parsing per-type** | 23 tests (SELECT, INSERT, UPDATE, DELETE, etc.) | 50+ tests (generic) | Need statement-type-specific tests |
| **Tab completion per-context** | 20+ tests (per CQL statement type) | 17 tests (generic contexts) | Need per-statement completion tests |
| **Float/numeric formatting** | 3+ tests | 1 test | Need more precision tests |
| **Date/time formatting** | 2+ tests | 0 integration | Need timestamp format tests |

---

## Implementation Plan

### Phase 1: Integration Test Infrastructure

| # | Task | Description | Priority |
|---|------|-------------|----------|
| 1.1 | Set up testcontainers-rs | Create `tests/integration/` with ScyllaDB container setup | P0 |
| 1.2 | Create shared test helpers | `setup_test_keyspace()`, `execute_cqlsh()`, `compare_output()` | P0 |
| 1.3 | CI integration | Add integration test job to GitHub Actions | P0 |

### Phase 2: Core Integration Tests (mirrors dtest TestCqlshSmoke)

| # | Task | Description | Python Equivalent |
|---|------|-------------|-------------------|
| 2.1 | Basic CRUD tests | SELECT, INSERT, UPDATE, DELETE via `-e` flag | `test_select`, `test_insert`, `test_update`, `test_delete` |
| 2.2 | DDL tests | CREATE/DROP KEYSPACE, TABLE, INDEX | `test_create_keyspace`, `test_drop_table`, etc. |
| 2.3 | USE command test | USE keyspace, verify prompt changes | `test_use_keyspace` |
| 2.4 | BATCH test | BATCH statement execution | `test_batch` |
| 2.5 | UUID test | UUID type round trip | `test_uuid` |
| 2.6 | TRUNCATE test | TRUNCATE with row verification | `test_truncate` |

### Phase 3: Output Formatting Tests (mirrors test_cqlsh_output.py)

| # | Task | Description | Python Equivalent |
|---|------|-------------|-------------------|
| 3.1 | No-color output test | Verify no ANSI codes with `--no-color` | `test_no_color_output` |
| 3.2 | Color output test | Verify ANSI codes with `-C` | `test_color_output` |
| 3.3 | Numeric output tests | Int, float, double, decimal display | `test_numeric_output` |
| 3.4 | Timestamp output test | Timestamp formatting | `test_timestamp_output` |
| 3.5 | Boolean output test | True/False display | `test_boolean_output` |
| 3.6 | NULL output test | Null value display | `test_null_output` |
| 3.7 | String output tests | ASCII and UTF-8 strings | `test_string_output_ascii`, `test_string_output_utf8` |
| 3.8 | Blob output test | Hex blob display | `test_blob_output` |
| 3.9 | Prompt test | Prompt format with keyspace/user | `test_prompt` |
| 3.10 | DESCRIBE output tests | DESCRIBE KEYSPACE, TABLE, CLUSTER output | `test_describe_*_output` |
| 3.11 | SHOW output test | SHOW VERSION, HOST | `test_show_output` |
| 3.12 | HELP output test | HELP text content | `test_help`, `test_help_types` |
| 3.13 | Error output tests | Parse/lex error messages | `test_printing_parse_error`, `test_printing_lex_error` |
| 3.14 | Multiline test | Multi-line statement handling | `test_multiline_statements` |
| 3.15 | EOF/exit test | Ctrl-D and EXIT behavior | `test_eof_prints_newline`, `test_exit_prints_no_newline` |

### Phase 4: Escape Sequence Tests (mirrors test_escape_*.py)

| # | Task | Description | Python Equivalent |
|---|------|-------------|-------------------|
| 4.1 | Hex escape decoding | `\x00`-`\xFF` decoding | `test_hex_escape_sequences` |
| 4.2 | Standard escape decoding | `\n`, `\t`, `\r`, etc. | `test_standard_escape_sequences` |
| 4.3 | Backslash escaping | Literal backslash handling | `test_backslash_escaping` |
| 4.4 | Quote escaping | Quote chars in strings | `test_quote_escaping` |
| 4.5 | Control char round-trip | Insert + select control chars | `test_control_chars_roundtrip` |
| 4.6 | Mixed content round-trip | Mixed escape sequences | `test_mixed_content_roundtrip` |

### Phase 5: Unicode Tests (mirrors test_unicode.py)

| # | Task | Description | Python Equivalent |
|---|------|-------------|-------------------|
| 5.1 | Unicode value round-trip | Insert + select Unicode values | `test_unicode_value_round_trip` |
| 5.2 | Unicode identifiers | Unicode in table/column names | `test_unicode_identifier` |
| 5.3 | Unicode multiline | Multi-line Unicode input | `test_unicode_multiline_input` |
| 5.4 | Unicode DESCRIBE | DESCRIBE with Unicode names | `test_unicode_desc` |

### Phase 6: COPY Integration Tests (mirrors cqlsh_copy_tests.py)

| # | Task | Description | Python Equivalent |
|---|------|-------------|-------------------|
| 6.1 | COPY TO basic | Export and verify CSV | `test_copy_to` (dtest) |
| 6.2 | COPY FROM basic | Import CSV and verify | `test_reading_use_header` |
| 6.3 | COPY round-trip | Export → truncate → import → verify | `test_all_datatypes_round_trip` |
| 6.4 | COPY with all types | All CQL data types | `test_all_datatypes_write`, `test_all_datatypes_read` |
| 6.5 | COPY with collections | List, set, map, tuple | `test_list_data`, `test_tuple_data` |
| 6.6 | COPY with custom delimiters | Pipe, colon, letter | `test_colon_delimiter`, etc. |
| 6.7 | COPY with NULL indicator | Custom null string | `test_null_as_null_indicator` |
| 6.8 | COPY with column ordering | Explicit column order | `test_explicit_column_order_*` |
| 6.9 | COPY with quoted columns | Quoted identifiers | `test_quoted_column_names_*` |
| 6.10 | COPY with counters | Counter columns | `test_reading_counter` |
| 6.11 | COPY error handling | Parse/insert errors, max errors | `test_copy_from_with_*_failures` |
| 6.12 | COPY with TTL | TTL on import | `test_reading_with_ttl` |
| 6.13 | COPY wrong column count | Mismatched columns | `test_wrong_number_of_columns` |
| 6.14 | Bulk round-trip | Large dataset (1000+ rows) | `test_bulk_round_trip_default` |

### Phase 7: Tab Completion Tests (mirrors test_cqlsh_completion.py)

| # | Task | Description | Python Equivalent |
|---|------|-------------|-------------------|
| 7.1 | Complete in SELECT | Column names, *, FROM, WHERE | `test_complete_in_select` |
| 7.2 | Complete in INSERT | Table name, column names, VALUES | `test_complete_in_insert` |
| 7.3 | Complete in UPDATE | Table, SET, WHERE | `test_complete_in_update` |
| 7.4 | Complete in DELETE | Table, WHERE | `test_complete_in_delete` |
| 7.5 | Complete in CREATE TABLE | Types, options | `test_complete_in_create_table` |
| 7.6 | Complete in DESCRIBE | Sub-commands, names | `test_complete_in_describe` |
| 7.7 | Complete in DROP | Object types, names | `test_complete_in_drop` |
| 7.8 | Complete in ALTER | Table, keyspace, type | `test_complete_in_alter_*` |
| 7.9 | Complete in GRANT/REVOKE | Permissions, roles | `test_complete_in_grant`, `test_complete_in_revoke` |
| 7.10 | Complete in string literals | No completion inside strings | `test_complete_in_string_literals` |

### Phase 8: LOGIN Tests (mirrors TestCqlLogin)

| # | Task | Description | Python Equivalent |
|---|------|-------------|-------------------|
| 8.1 | LOGIN keeps keyspace | Keyspace preserved after LOGIN | `test_login_keeps_keyspace` |
| 8.2 | LOGIN rejects bad password | Error on wrong credentials | `test_login_rejects_bad_pass` |
| 8.3 | LOGIN correct user | Successful re-auth | `test_login_authenticates_correct_user` |
| 8.4 | LOGIN failure continues | Failed LOGIN doesn't disconnect | `test_login_allows_bad_pass_and_continued_use` |

---

## Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Total tests | 327 | 500+ |
| Integration tests | 0 | 80+ |
| Test categories covered | ~10 | 18+ |
| Output comparison tests | 0 | 25+ |
| COPY integration tests | 0 | 14+ |
| Escape sequence tests | 0 | 13+ |
| Unicode tests | 0 | 5+ |

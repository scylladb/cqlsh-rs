# Upstream PR Review: scylladb/scylla-cqlsh Open PRs

> Parent: [high-level-design.md](high-level-design.md)
> Date: 2026-03-14
> Source: https://github.com/scylladb/scylla-cqlsh/pulls
> **Status: COMPLETED** — PR #150 (parser fix) and PR #151 (O(n) parsing) incorporated into SP4. PR #154 not needed. PR #147 (safe mode) deferred to Phase 4. PR #163 (SSL tests) deferred to Phase 5.

## Purpose

This document captures open PRs from the upstream Python scylla-cqlsh that represent bug fixes, performance improvements, and new features. cqlsh-rs must account for these to avoid inheriting the same bugs and to implement equivalent features.

---

## PR Summary

### PR #150 — Fix parser treating `/*` in string literals as comment delimiter

**Issue:** SCYLLADB-341
**Impact:** Statement Parser (SP4)
**Severity:** Bug — correctness

**Problem:** The CQL parser incorrectly interprets `/*` within string literals as block comment delimiters, causing "Incomplete statement at end of file" errors.

```sql
-- This fails in unpatched Python cqlsh:
INSERT INTO system_auth.role_attributes (role, name, value)
VALUES ('test_role2', 'test_role./*', 'test');
```

**Root Cause:** The Python cqlsh used a regex-based `strip_comment_blocks()` preprocessing step that naively matched `/*` regardless of whether it appeared inside string literals.

**Fix (upstream):** Remove the flawed regex preprocessing and reorder lexer grammar rules so string literal patterns are matched **before** comment patterns (aligning with Apache Cassandra's upstream fix).

**cqlsh-rs Action Items:**
1. **Parser design (SP4 Step 6):** Ensure block comment stripping respects string literal context. The lexer/tokenizer must process tokens in order: string literals → comments → other tokens. Never use regex preprocessing on raw input.
2. **Test cases to add:**
   - `/*` inside single-quoted string values
   - `*/` inside string values
   - `/* */` spanning multiple lines with strings containing `/*`
   - Mixed `--` line comments and `/* */` block comments with string literals
   - Dollar-quoted strings (`$$`) containing `/*`

---

### PR #151 — Fix O(n²) performance issue for large UDF insertion

**Issue:** SCYLLADB-338
**Impact:** Statement Parser (SP4), Batch Mode (runner.rs)
**Severity:** Performance — critical for large inputs

**Problem:** Inserting large UDF files (>1MB, 32K+ lines) takes >2 hours because in batch mode (non-interactive), the Python cqlsh re-parses the entire accumulated statement buffer on every new line.

**Root Cause:** `cmdloop()` calls the parser on every line of input. For a single large statement (e.g., a CREATE FUNCTION with a massive WASM blob), this means O(n²) parsing where n is the number of lines.

**Fix (upstream):** In batch mode, only invoke the parser when a line ends with a semicolon. Otherwise, just accumulate the line into the buffer.

**cqlsh-rs Action Items:**
1. **Batch mode design (runner.rs / parser.rs):** The statement accumulator must NOT re-parse on every line. Use a simple state machine: accumulate lines, only attempt full parse when a potential statement terminator (`;`) is detected outside of string/comment context.
2. **Optimization:** Use an incremental approach — track whether we're inside a string literal or comment as lines are added, so semicolon detection is O(1) per line.
3. **Test cases to add:**
   - Benchmark: Parse a 1MB+ multi-line CQL statement (should complete in <1s)
   - Large CREATE FUNCTION with embedded WASM blob
   - Large batch statements (10K+ individual statements in a single file)
   - Verify that partial-line semicolons inside strings don't trigger premature parsing

---

### PR #154 — Remove obsolete DesDateType deletion hack

**Issue:** SCYLLADB-345
**Impact:** Driver Layer (SP2), Type System (SP9)
**Severity:** Cleanup — informational

**Problem:** Python cqlsh contained a conditional hack that deleted `cassandra.deserializers.DesDateType` to work around old driver compatibility issues. This is no longer needed with scylla-driver ≥3.29.7.

**cqlsh-rs Action Items:**
1. **No direct action needed** — cqlsh-rs uses the scylla Rust driver which has a clean type system. This PR confirms that date type deserialization quirks from the Python driver do not need to be replicated.
2. **Validation:** Ensure the scylla Rust driver handles `date` type correctly without any special workarounds.

---

### PR #147 — Add safe mode with confirmation prompts for DROP/TRUNCATE

**Issue:** SCYLLADB-342
**Impact:** CLI & Config (SP1), Built-in Commands (SP7), Statement Pipeline
**Severity:** New Feature

**Problem:** Users can accidentally execute destructive operations (DROP KEYSPACE, TRUNCATE, etc.) without any safety net.

**Feature:** An opt-in `--safe-mode` flag (also configurable via cqlshrc) that prompts "Are you sure you want to [OPERATION] [TARGET]? [N/y]" before executing destructive statements.

**Protected operations:**
- `DROP KEYSPACE`, `DROP TABLE`, `DROP INDEX`, `DROP MATERIALIZED VIEW`
- `DROP TYPE`, `DROP FUNCTION`, `DROP AGGREGATE`
- `DROP USER`, `DROP ROLE`, `DROP SERVICE_LEVEL`, `DROP TRIGGER`
- `TRUNCATE`

**Implementation details (from upstream):**
- Uses whitespace-normalized regex matching: `re.sub(r'\s+', ' ', statement.strip())`
- Default answer is No (safe default)
- Skips prompts in non-interactive mode (batch/execute mode)
- Configurable via `[connection]` section in cqlshrc: `safe_mode = true`

**cqlsh-rs Action Items:**
1. **CLI flag (SP1):** Add `--safe-mode` flag to `CliArgs` struct
2. **Config (SP1):** Add `safe_mode` to cqlshrc `[connection]` section parsing
3. **Statement pipeline:** After parsing a complete statement, before execution, check if safe mode is enabled and the statement matches a destructive pattern. If so, prompt for confirmation.
4. **Implementation approach:**
   - Parse the statement's first tokens to detect DROP/TRUNCATE
   - Use the statement parser's existing tokenization (not regex on raw input)
   - Respect non-interactive mode (skip prompts when stdin is not a TTY)
5. **Test cases to add:**
   - Each protected operation triggers a prompt in safe mode
   - Confirmation with 'y'/'Y' proceeds
   - Any other input (including empty/Enter) cancels
   - Non-interactive mode skips prompts
   - Safe mode off (default) does not prompt
   - Whitespace variations in statements still trigger detection

---

### PR #163 — SSL/TLS integration tests with testcontainers

**Issue:** N/A (testing infrastructure)
**Impact:** Testing Strategy (SP10), Driver & Connection (SP2)
**Severity:** Testing infrastructure

**Problem:** The upstream Python cqlsh lacked integration tests for SSL/TLS functionality.

**Test coverage added (39 tests):**
- Certificate generation and validation utilities
- Basic SSL connections
- Mutual TLS (client authentication)
- SSL configuration via CLI flags, cqlshrc, and environment variables
- CQL operations over SSL (DML, DDL, COPY, batch)
- TLS version enforcement (≥1.2)
- Error handling (expired certs, wrong CA, hostname mismatch)

**cqlsh-rs Action Items:**
1. **Testing strategy (SP10):** Add equivalent SSL/TLS integration test suite. Key test categories:
   - Connection with server-only TLS
   - Mutual TLS with client certificates
   - Certificate validation (valid CA, invalid CA, expired cert, hostname mismatch)
   - Configuration precedence for SSL settings (CLI `--ssl` > env `SSL_CERTFILE` > cqlshrc `[ssl]`)
   - CQL operations work correctly over encrypted connections
   - TLS version negotiation (reject TLS <1.2)
2. **Test infrastructure:** Use `rcgen` crate for test certificate generation (Rust equivalent of Python's `cryptography` library)
3. **SP2 validation:** SSL/TLS acceptance criteria already include "SSL/TLS connections work with all cert options from cqlshrc" — these tests help validate that

---

## Impact Matrix

| PR | SP1 (CLI) | SP2 (Driver) | SP4 (Parser) | SP7 (Commands) | SP9 (Types) | SP10 (Testing) | SP11 (Bench) |
|----|-----------|-------------|-------------|---------------|------------|----------------|-------------|
| #150 Parser fix | | | **Critical** | | | Tests | |
| #151 O(n²) perf | | | **Critical** | | | Tests | **Benchmark** |
| #154 DesDateType | | Informational | | | Informational | | |
| #147 Safe mode | **New flag** | | | **New feature** | | Tests | |
| #163 SSL tests | | Validation | | | | **New tests** | |

---

## Integration into Development Plan

### Immediate Actions (Before Implementation)

1. **SP4 (Statement Parser):**
   - Add acceptance criterion: "Block comments inside string literals do NOT terminate or split statements"
   - Add acceptance criterion: "Batch mode parsing is O(n) not O(n²) — verified by benchmark"
   - Add implementation note: "Tokenize in order: string literals → comments → other tokens (never regex-preprocess raw input)"

2. **SP1 (CLI & Config):**
   - Add `--safe-mode` to CLI flag table (Priority: P3)
   - Add `safe_mode` to `[connection]` cqlshrc section

3. **SP7 (Built-in Commands):**
   - Add "Safe mode confirmation" as a cross-cutting concern in the statement execution pipeline

4. **SP10 (Testing):**
   - Add SSL/TLS integration test suite to test plan
   - Add parser edge case tests (comment-in-string, large input performance)
   - Add safe mode unit and integration tests

5. **SP11 (Benchmarking):**
   - Add "Large statement parsing" benchmark (target: 1MB statement in <1s)

### Compatibility Matrix Additions

Add to high-level-design.md:

| Flag | Short | Description | Priority | Source |
|------|-------|-------------|----------|--------|
| `--safe-mode` | | Prompt before DROP/TRUNCATE operations | P3 | scylla-cqlsh PR #147 |

| cqlshrc Key | Section | Description | Priority | Source |
|-------------|---------|-------------|----------|--------|
| `safe_mode` | `[connection]` | Enable safe mode (true/false) | P3 | scylla-cqlsh PR #147 |

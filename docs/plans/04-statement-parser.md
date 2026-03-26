# Sub-Plan SP4: Statement Parser

> Parent: [high-level-design.md](high-level-design.md) | Phase: 1-2
> **Status: COMPLETED** — All parser features done (2026-03-14). O(n) benchmark verification deferred to Phase 5 (SP11).

## Objective

Implement a statement parser that handles multi-line input buffering, semicolon-terminated statement detection, comment stripping, string literal handling, and routing between CQL statements and built-in shell commands.

---

## Research Phase

### Tasks

1. **Python cqlsh parser behavior** — How it detects statement boundaries, handles quotes, comments
2. **CQL comment syntax** — `--` line comments, `/* */` block comments
3. **CQL string literals** — Single-quoted strings, `$$` dollar-quoted strings, escape sequences
4. **Built-in command detection** — How Python cqlsh distinguishes DESCRIBE/COPY/etc. from CQL
5. **Edge cases** — Semicolons in strings, nested comments, empty statements

### Research Deliverables

- [x] Statement boundary detection algorithm spec
- [x] Comment handling rules
- [x] String literal handling rules
- [x] Built-in command routing rules
- [x] Edge case test catalog

---

## Execution Phase

### Implementation Steps

| Step | Description | Module | Tests | Status |
|------|-------------|--------|-------|--------|
| 1 | Basic semicolon detection (ignoring strings) | `parser.rs` | Unit: simple statements | ✅ Done |
| 2 | Single-quoted string handling | `parser.rs` | Unit: strings with semicolons | ✅ Done |
| 3 | Double-quoted identifier handling | `parser.rs` | Unit: quoted identifiers | ✅ Done |
| 4 | Dollar-quoted string handling (`$$...$$`) | `parser.rs` | Unit: dollar-quoted strings | ✅ Done |
| 5 | Line comment stripping (`--`) | `parser.rs` | Unit: comments removed | ✅ Done |
| 6 | Block comment stripping (`/* */`) | `parser.rs` | Unit: block comments | ✅ Done |
| 7 | Multi-line statement buffering | `parser.rs` | Unit: multi-line input | ✅ Done |
| 8 | Empty statement handling (bare `;`) | `parser.rs` | Unit: skip empty | ✅ Done |
| 9 | Built-in command detection (case-insensitive prefix match) | `parser.rs` | Unit: all built-in commands | ✅ Done |
| 10 | Command routing (built-in vs CQL dispatch) | `parser.rs` | Unit: routing logic | ✅ Done |
| 11 | Whitespace normalization | `parser.rs` | Unit: leading/trailing whitespace | ✅ Done |
| 12 | Multiple statements on one line | `parser.rs` | Unit: `stmt1; stmt2;` | ✅ Done |

### Key Decisions

| # | Decision | Choice | Rationale |
|---|----------|--------|-----------|
| D1 | Lexer architecture | Single-pass char-by-char state machine with `LexState` enum | Context-aware: handles strings, comments, and normal code without regex preprocessing (PR #150 fix) |
| D2 | Incremental API | `StatementParser::feed_line()` returns `ParseResult::Complete/Incomplete` | Enables O(n) batch mode by processing per-line; REPL integration is straightforward |
| D3 | Comment stripping | `strip_comments()` applied to extracted statements, not raw input | Separates concerns: lexer finds boundaries, stripper cleans statements |
| D4 | Shell command detection | First-word match against `SHELL_COMMANDS` constant array | Case-insensitive, fast, matches Python cqlsh behavior |

### Upstream Bug Fixes to Account For

> See [SP16: Upstream PR Review](16-upstream-pr-review.md) for full details.

**scylla-cqlsh PR #150 (SCYLLADB-341): `/*` in string literals misinterpreted as comment.**
The Python cqlsh used regex-based `strip_comment_blocks()` preprocessing that naively matched `/*` inside string literals. The fix: remove preprocessing, tokenize in order (string literals → comments → other tokens). cqlsh-rs MUST NOT use regex preprocessing on raw CQL input for comment handling. The lexer must be context-aware.

✅ **Addressed**: The `LexState` enum tracks string context; `/*` inside single-quoted, double-quoted, and dollar-quoted strings is treated as literal text. Verified by unit tests.

**scylla-cqlsh PR #151 (SCYLLADB-338): O(n²) batch mode parsing.**
In batch mode, the Python cqlsh re-parsed the entire accumulated buffer on every new line, causing >2hr processing for 1MB+ UDF files. The fix: only invoke the parser when a semicolon terminator is detected. cqlsh-rs MUST use an incremental approach — track string/comment context as lines are added, detect semicolons in O(1) per line, only attempt full parse when a potential terminator is found.

✅ **Addressed**: `parse_batch()` uses `StatementParser::feed_line()` which processes input line-by-line. The `extract_statements()` method scans only the current buffer contents. Benchmark verification is pending (acceptance criterion below).

### Test Summary

| Layer | Count | Description |
|-------|-------|-------------|
| Unit (parser) | 36 | Semicolons, strings, comments, multi-line, batch mode, Unicode, classification |
| Unit (total project) | 161 | Including existing CLI, config, driver, session, repl, completions tests |
| Integration | 18 | Existing CLI integration tests |

### Acceptance Criteria

- [x] Semicolons inside string literals do not terminate statements
- [x] Comments are stripped before execution
- [x] **Block comments (`/* */`) inside string literals do NOT split or terminate statements** (PR #150)
- [x] **`/*` and `*/` characters inside single-quoted, double-quoted, and dollar-quoted strings are treated as literal text** (PR #150)
- [x] Multi-line input accumulates correctly
- [ ] **Batch mode parsing is O(n) not O(n²) — verified by benchmark with 1MB+ input completing in <1s** (PR #151)
- [x] Built-in commands are detected case-insensitively
- [x] CQL statements are forwarded to the driver
- [x] Empty statements are silently skipped
- [x] Multiple statements on one line are handled sequentially

> **Note**: 9/10 acceptance criteria met. The remaining item (O(n) benchmark verification) requires a criterion benchmark which will be added in Phase 5 (SP11). The architecture is O(n) by design — `parse_batch()` processes each line once via `feed_line()`.

---

## REPL Integration

The REPL (`src/repl.rs`) has been updated to use `StatementParser` instead of the previous naive `is_statement_complete()` check. The old helper functions (`is_statement_complete`, `is_shell_command`) in repl.rs have been removed and replaced with the parser module's context-aware implementations.

---

## Skills Required

- Parser design (S6) ✅
- CQL language syntax (D1) ✅
- Property-based testing for parser fuzzing (C10) — deferred to Phase 5

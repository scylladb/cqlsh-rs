# Sub-Plan SP20: Grammar-Aware Tab Completion

> Parent: [high-level-design.md](high-level-design.md) | Phase 3 (enhancement)
>
> **This is a living document.** Update it as development progresses.
> Fixes: [#86](https://github.com/scylladb/cqlsh-rs/issues/86), [#85](https://github.com/scylladb/cqlsh-rs/issues/85)

## Objective

Make tab completion grammar-aware: only suggest tokens that are syntactically valid at the current cursor position. Never suggest a token that would produce a CQL syntax error.

---

## Relationship to PR #51 (SP18: Unified CQL Lexer)

PR #51 introduced the unified CQL lexer (`src/cql_lexer.rs`) with `GrammarContext` tracking. This is **foundational infrastructure** that SP20 builds on top of. PR #51 should be **merged first** — it provides the lexer, token types, and grammar context enum that this plan extends.

**SP20 does NOT replace PR #51.** It enhances the `GrammarContext` state machine and the `completer.rs` context-to-completion mapping to be more precise.

---

## Requirements & Constraints

| ID | Type | Description |
|----|------|-------------|
| REQ-01 | Requirement | `SELECT ⇥` must offer `*`, `DISTINCT`, column names, function names — NOT all clause keywords |
| REQ-02 | Requirement | `SELECT * FROM ks.⇥` must offer tables in `ks` — NOT clause keywords |
| REQ-03 | Requirement | `CREATE ⇥` must offer `TABLE`, `KEYSPACE`, `INDEX`, `TYPE`, `FUNCTION`, `AGGREGATE`, `MATERIALIZED` — NOT all keywords |
| REQ-04 | Requirement | `INSERT INTO ⇥` must offer keyspace/table names — NOT keywords |
| REQ-05 | Requirement | `ALTER ⇥` must offer `TABLE`, `KEYSPACE`, `TYPE`, `ROLE`, `USER` — NOT all keywords |
| REQ-06 | Requirement | `DROP ⇥` must offer `TABLE`, `KEYSPACE`, `INDEX`, `TYPE`, `FUNCTION`, `AGGREGATE`, `MATERIALIZED`, `TRIGGER`, `ROLE`, `USER` |
| REQ-07 | Requirement | `DELETE ⇥` must offer column names (if table known) or `FROM` |
| REQ-08 | Requirement | `GRANT ⇥` / `REVOKE ⇥` must offer permission names |
| REQ-09 | Constraint | Completion latency must remain <50ms |
| REQ-10 | Constraint | Must not break existing colorizer or parser that share the lexer |
| CON-01 | Constraint | Build on top of PR #51's `GrammarContext` enum and lexer infrastructure |

## Design Decisions

| Decision | Choice | Rationale | Alternatives Rejected |
|----------|--------|-----------|----------------------|
| Approach | Extend `GrammarContext` with statement-type-aware variants | Minimal change to existing architecture; completer already dispatches on `GrammarContext` | Full CQL parser (overkill), separate completion grammar (duplication) |
| Where to fix | Both `cql_lexer.rs` (context detection) and `completer.rs` (context-to-candidates mapping) | Context detection must be more precise, AND the candidate lists must be per-statement-type | Only fixing completer (wouldn't fix the `ks.table` bug) |
| Keyword lists | Per-statement-type keyword lists in `completer.rs` | Each statement type has a small, well-defined set of valid next tokens | Single flat keyword list (current — broken) |

---

## Bug Analysis

### Bug #85: `SELECT * FROM system.c⇥` shows keywords instead of tables

**Root cause:** In `grammar_context_at_end()`, when the last significant token is an identifier (`system`) followed by `.` followed by partial identifier `c`, the function sees `c` as the last token. Since `c` isn't preceded by a recognized keyword pattern, it falls through to `GrammarContext::General`. The completer maps `General` to `ClauseKeyword`, which dumps all clause keywords.

**Fix:** The function correctly returns `ExpectQualifiedPart` when the last token is `.`, but when the user has started typing after the dot (e.g., `system.c`), the last token is `c` (an identifier), and the dot is second-to-last. The function must recognize: if second-to-last is `.` AND there's a table-expecting keyword earlier in the stream, the context is `ExpectTable { keyspace: Some("system") }`.

### Bug #86: `SELECT ⇥` shows all clause keywords

**Root cause:** `completer.rs` line 320-326 — `ExpectColumnList` context falls through to `CompletionContext::ClauseKeyword` when there's a space after SELECT. This dumps all 70+ clause keywords.

**Fix:** `ExpectColumnList` should map to a dedicated candidate list: `*`, `DISTINCT`, `JSON`, column names (if table known from prior context), and function names like `count(`, `now(`, `writetime(`.

---

## Implementation Tasks

### Phase 1: Fix `grammar_context_at_end` for qualified names (fixes #85)

| # | Task | Description | Validation |
|---|------|-------------|------------|
| 1.1 | Fix qualified name context in `context_from_tokens()` | When last token is an identifier and second-to-last is `.`, look further back to determine if we're in a table-expecting context (FROM/INTO/UPDATE/TABLE before the `ks.` pattern). Return `ExpectQualifiedPart` with enough info for completer to resolve. | Unit test: `grammar_context_at_end("SELECT * FROM system.c")` returns `ExpectQualifiedPart` |
| 1.2 | Fix `detect_context()` in `completer.rs` for `ExpectQualifiedPart` | When in `ExpectQualifiedPart`, the `extract_qualifying_keyspace` must handle the case where tokens are `[..., "system", ".", "c"]` (dot is NOT last). Extract keyspace from token before the dot. | Unit test: `detect_context("SELECT * FROM system.c", 22)` returns `TableName { keyspace: Some("system") }` |
| 1.3 | Manual test | `SELECT * FROM system.c⇥` shows `clients`, `compaction_history`, etc. | Manual verification against running ScyllaDB |

### Phase 2: Statement-type-aware completion (fixes #86)

| # | Task | Description | Validation |
|---|------|-------------|------------|
| 2.1 | Add `ExpectSelectTarget` variant or reuse `ExpectColumnList` properly | Map `ExpectColumnList` in completer to offer: `*`, `DISTINCT`, `JSON`, column names (if table known), built-in functions (`count(`, `writetime(`, `ttl(`, `now()`, `uuid()`, `toJson(`) | Unit test: `complete("SELECT ", 7)` returns `*`, `DISTINCT`, `JSON` and NOT `ADD`, `COMPACT`, etc. |
| 2.2 | Add `ExpectCreateTarget` grammar context | After `CREATE`, the valid next tokens are: `TABLE`, `KEYSPACE`, `INDEX`, `TYPE`, `FUNCTION`, `AGGREGATE`, `MATERIALIZED`, `TRIGGER`, `ROLE`, `USER`, `CUSTOM` | Unit test: `grammar_context_at_end("CREATE ")` returns `ExpectCreateTarget`; completer offers only valid sub-commands |
| 2.3 | Add `ExpectAlterTarget` grammar context | After `ALTER`: `TABLE`, `KEYSPACE`, `TYPE`, `ROLE`, `USER`, `MATERIALIZED` | Unit test |
| 2.4 | Add `ExpectDropTarget` grammar context | After `DROP`: `TABLE`, `KEYSPACE`, `INDEX`, `TYPE`, `FUNCTION`, `AGGREGATE`, `MATERIALIZED`, `TRIGGER`, `ROLE`, `USER` | Unit test |
| 2.5 | Add `ExpectDeleteTarget` grammar context | After `DELETE` (before FROM): column names or `FROM` | Unit test |
| 2.6 | Add `ExpectGrantRevoke` grammar context | After `GRANT`/`REVOKE`: permission names (`ALL`, `ALTER`, `AUTHORIZE`, `CREATE`, `DESCRIBE`, `DROP`, `EXECUTE`, `MODIFY`, `SELECT`) | Unit test |
| 2.7 | Add `ExpectInsertTarget` grammar context | After `INSERT`: only `INTO` | Unit test |
| 2.8 | Update `context_from_tokens()` in `cql_lexer.rs` | Add match arms for `CREATE`, `ALTER`, `DROP`, `DELETE`, `GRANT`, `REVOKE`, `INSERT` to return the new context variants | All new unit tests pass |
| 2.9 | Update `is_strict_identifier_context()` | Ensure new contexts that expect identifiers are included | Existing colorizer tests still pass |
| 2.10 | Update `CompletionContext` enum and `complete_for_context()` in `completer.rs` | Add per-statement-type candidate lists. Each new `GrammarContext` maps to a `CompletionContext` with a curated keyword list. | Unit tests for each context |

### Phase 3: Testing & validation

| # | Task | Description | Validation |
|---|------|-------------|------------|
| 3.1 | Add comprehensive unit tests | Test every statement type + position combination from the requirements table | `cargo test` passes, all new tests green |
| 3.2 | Add negative tests | Verify that invalid suggestions are NOT offered (e.g., `SELECT ⇥` must NOT contain `ADD`, `COMPACT`, `STORAGE`) | Negative assertions in unit tests |
| 3.3 | Regression tests | Ensure all existing completer tests still pass; run full test suite | `cargo test` — zero regressions |
| 3.4 | Manual testing | Test all 9 groups from the manual test plan against running ScyllaDB | Manual verification documented |
| 3.5 | Update SP05 and SP18 docs | Mark as enhanced by SP20, cross-reference | Docs updated |

---

## Expected Completion Context Table

This is the target state — every position should offer ONLY these candidates:

| Input context | Valid completions |
|---|---|
| `⇥` (empty) | Statement keywords (`SELECT`, `INSERT`, `CREATE`, ...) + shell commands |
| `SELECT ⇥` | `*`, `DISTINCT`, `JSON`, column names, functions |
| `SELECT * FROM ⇥` | keyspace names, table names |
| `SELECT * FROM ks.⇥` | table names in `ks` |
| `SELECT * FROM ks.t⇥` | table names in `ks` starting with `t` |
| `SELECT * FROM tbl WHERE ⇥` | column names of `tbl` |
| `CREATE ⇥` | `TABLE`, `KEYSPACE`, `INDEX`, `TYPE`, `FUNCTION`, `AGGREGATE`, `MATERIALIZED`, `TRIGGER`, `ROLE`, `USER`, `CUSTOM` |
| `ALTER ⇥` | `TABLE`, `KEYSPACE`, `TYPE`, `ROLE`, `USER`, `MATERIALIZED` |
| `DROP ⇥` | `TABLE`, `KEYSPACE`, `INDEX`, `TYPE`, `FUNCTION`, `AGGREGATE`, `MATERIALIZED`, `TRIGGER`, `ROLE`, `USER` |
| `INSERT ⇥` | `INTO` |
| `INSERT INTO ⇥` | keyspace/table names |
| `DELETE ⇥` | `FROM`, column names |
| `GRANT ⇥` | `ALL`, `ALTER`, `AUTHORIZE`, `CREATE`, `DESCRIBE`, `DROP`, `EXECUTE`, `MODIFY`, `SELECT` |
| `REVOKE ⇥` | same as GRANT |
| `USE ⇥` | keyspace names |
| `CONSISTENCY ⇥` | consistency levels |
| `DESCRIBE ⇥` | describe sub-commands |
| `DESCRIBE TABLE ⇥` | table names |
| `DESCRIBE KEYSPACE ⇥` | keyspace names |
| `SOURCE ⇥` | file paths |
| `UPDATE ⇥` | keyspace/table names |
| `UPDATE tbl SET ⇥` | column names of `tbl` |
| `TRUNCATE ⇥` | keyspace/table names |
| `BEGIN ⇥` | `BATCH`, `UNLOGGED`, `COUNTER` |
| `SELECT * FROM tbl ORDER BY ⇥` | column names of `tbl` |
| `SELECT * FROM tbl LIMIT ⇥` | (no keyword completion — expects number) |

---

## Dependencies

| ID | Dependency | Required By |
|----|-----------|-------------|
| DEP-01 | PR #51 (SP18 unified CQL lexer) merged | All tasks — we extend its `GrammarContext` |

## Risks

| ID | Risk | Mitigation |
|----|------|-----------|
| RISK-01 | New `GrammarContext` variants break colorizer | Run all existing tests after each change; `is_strict_identifier_context` must be updated |
| RISK-02 | Grammar edge cases (subqueries, nested functions) | Start with top-level statement contexts; handle nested contexts in follow-up |
| RISK-03 | Performance regression from more complex context detection | `context_from_tokens` is O(n) on token count; adding more match arms is negligible |

## Open Questions

| # | Question | Status | Decision |
|---|----------|--------|----------|
| 1 | Should we complete function names (count, now, uuid, etc.)? | Open | Likely yes for SELECT context |
| 2 | Should we handle subquery contexts (SELECT ... FROM (SELECT ...))? | Open | Defer — rare in interactive use |
| 3 | Should `BEGIN ⇥` offer `BATCH`, `UNLOGGED`, `COUNTER`? | Open | Yes if Python cqlsh does |

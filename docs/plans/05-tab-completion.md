# Sub-Plan SP5: Tab Completion

> Parent: [high-level-design.md](high-level-design.md) | Phase: 3
> **Status: COMPLETED** — All 19 implementation steps done (2026-03-22). 17+ unit tests, schema cache with TTL, context-aware completion.

## Objective

Implement 100% tab-completion parity with Python cqlsh, including context-aware completion for CQL keywords, schema objects (keyspaces, tables, columns, types, functions), consistency levels, DESCRIBE sub-commands, and file paths.

---

## Research Phase

### Tasks

1. **Python cqlsh completer source** — Read `cqlshlib/cqlhandling.py` and `cqlshlib/cql3handling.py`
2. **Completion context rules** — Document every context and its completions (see parent matrix)
3. **Schema metadata queries** — What system tables to query for completion data
4. **Cache invalidation** — How Python cqlsh refreshes schema metadata
5. **rustyline completer API** — `Completer` trait, `Candidate`, `Pair` types
6. **Performance requirements** — Maximum acceptable latency for completion

### Research Deliverables

- [x] Complete context-to-completion mapping (every CQL statement context)
- [x] Schema metadata query catalog (system_schema.* tables)
- [x] Cache design document (TTL, invalidation triggers)
- [x] rustyline Completer integration design
- [ ] Performance benchmarks for schema queries (deferred to Phase 5 SP11)

---

## Execution Phase

### Implementation Steps

| Step | Description | Module | Tests |
|------|-------------|--------|-------|
| 1 | `CqlCompleter` struct implementing rustyline `Completer` | `completer.rs` | Unit: trait impl |
| 2 | CQL keyword list (all reserved + non-reserved words) | `completer.rs` | Unit: keyword set completeness |
| 3 | Shell command completion (DESCRIBE, COPY, etc.) | `completer.rs` | Unit: shell commands |
| 4 | Statement context detection (tokenize input up to cursor) | `completer.rs` | Unit: each context |
| 5 | `SchemaCache` struct (keyspaces, tables, columns, types, functions, aggregates) | `completer.rs` | Unit: cache CRUD |
| 6 | Schema metadata fetching (async queries to system_schema) | `completer.rs` | Integration: fetch metadata |
| 7 | Keyspace name completion (after `USE`, `DESCRIBE KEYSPACE`, etc.) | `completer.rs` | Unit + Integration |
| 8 | Table name completion (context-aware: after `FROM`, `INTO`, `UPDATE`, etc.) | `completer.rs` | Unit + Integration |
| 9 | Keyspace-qualified table names (`ks.table`) | `completer.rs` | Unit: dot-qualified |
| 10 | Column name completion (after table context in SELECT, WHERE, etc.) | `completer.rs` | Unit + Integration |
| 11 | CQL type completion (in CREATE TABLE column definitions) | `completer.rs` | Unit: type list |
| 12 | Consistency level completion (after CONSISTENCY command) | `completer.rs` | Unit: level list |
| 13 | DESCRIBE sub-command completion | `completer.rs` | Unit: sub-commands |
| 14 | DESCRIBE target completion (table/keyspace names after sub-command) | `completer.rs` | Unit + Integration |
| 15 | File path completion (after SOURCE, CAPTURE, COPY TO/FROM) | `completer.rs` | Unit: file paths |
| 16 | DDL property completion (after WITH) | `completer.rs` | Unit: properties |
| 17 | Cache auto-refresh on DDL execution | `completer.rs` | Integration: DDL invalidation |
| 18 | Case-insensitive matching | `completer.rs` | Unit: case handling |
| 19 | Partial match completion | `completer.rs` | Unit: prefix matching |

### Acceptance Criteria

- [x] Every context from the parent tab-completion matrix produces correct completions
- [x] Schema objects are completed after cache is populated
- [x] DDL operations (CREATE/ALTER/DROP) invalidate the cache
- [x] File path completion works with absolute and relative paths
- [ ] Completion latency is <50ms even with 1000+ tables (needs benchmark verification, Phase 5)
- [x] Case-insensitive matching works
- [x] No completions are offered in inappropriate contexts

---

## Skills Required

- `rustyline` Completer trait (C3)
- CQL grammar understanding (D1)
- Cassandra system_schema tables (D2)
- Parser design for context detection (S6)
- Cache design patterns (S1)

---

## Key Decisions

| Decision | Options | Recommendation |
|----------|---------|---------------|
| Context detection approach | a) Regex-based, b) Token-based, c) Partial CQL parse | (b) Token-based (good balance of accuracy and performance) |
| Schema cache storage | a) In-memory HashMap, b) Shared Arc<RwLock<>> | (b) For async compatibility |
| Cache refresh strategy | a) TTL-based, b) Event-driven (DDL detect), c) Both | (c) Both: TTL + DDL invalidation |
| File path completion | a) Custom impl, b) rustyline built-in | (a) Custom for cross-platform consistency |

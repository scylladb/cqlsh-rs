# Sub-Plan: Phase 4 — COPY & Advanced

> Parent: [high-level-design.md](high-level-design.md) | Phase 4
>
> **This is a living document.** Update it as development progresses.

## Objective

Achieve 100% feature parity with Python cqlsh by implementing COPY TO/FROM, remaining DESCRIBE variants, non-interactive batch mode, and integrating all remaining CLI flags.

---

## Requirements & Constraints

| ID | Type | Description |
|----|------|-------------|
| REQ-01 | Requirement | COPY TO exports all CQL data types to CSV with all 17 options |
| REQ-02 | Requirement | COPY FROM imports CSV with all 19 options, parallel workers, rate limiting |
| REQ-03 | Requirement | DESCRIBE covers all schema objects: INDEX, VIEW, TYPE, FUNCTION, AGGREGATE |
| REQ-04 | Requirement | Non-interactive mode (`-e`, `-f`) works with proper exit codes |
| REQ-05 | Requirement | LOGIN command re-authenticates without restarting |
| REQ-06 | Constraint | Output format matches Python cqlsh exactly (CSV escaping, progress reports) |
| REQ-07 | Constraint | All 24 CLI flags functional (not just parsed) |
| GUD-01 | Guideline | Use Tokio tasks for parallelism (consistent with existing architecture) |
| GUD-02 | Guideline | Use `csv` crate for CSV parsing/writing |

## Design Decisions

| Decision | Choice | Rationale | Alternatives Rejected |
|----------|--------|-----------|----------------------|
| COPY parallelism | Tokio async tasks | Consistent with app; no OS thread overhead | multiprocessing (Python approach) |
| CSV library | `csv` crate | Mature, well-tested, handles edge cases | manual parsing |
| Rate limiting | Token bucket | Smooth throughput control | simple sleep-per-batch |
| Batch insertion | Individual prepared statements | Matches Python cqlsh behavior | CQL BATCH (different semantics) |
| COPY module location | `src/copy.rs` | Standalone module, not nested under commands/ | commands/copy.rs |

---

## Implementation Tasks

### Phase 4.1: Non-Interactive Mode & CLI Flags (Foundation)

These unblock testing and scripting for all subsequent work.

| # | Task | Description | Files | Validation |
|---|------|-------------|-------|------------|
| 4.1.1 | Implement `-e` execution mode | Execute CQL string from `--execute` flag, print results, exit with status code | `src/main.rs`, `src/repl.rs` | `cqlsh-rs -e "SELECT * FROM system.local"` produces tabular output and exits 0 |
| 4.1.2 | Implement `-f` file execution mode | Execute CQL from file via `--file` flag, reuse SOURCE logic | `src/main.rs`, `src/repl.rs` | `cqlsh-rs -f script.cql` executes all statements and exits |
| 4.1.3 | Exit codes for batch mode | Exit 0 on success, 1 on CQL error, 2 on connection failure | `src/main.rs` | `cqlsh-rs -e "INVALID"; echo $?` prints `1` |
| 4.1.4 | Integrate `--tty` flag | Force TTY behavior (pager, color) even when piped | `src/repl.rs` | `cqlsh-rs --tty -e "SELECT..." \| cat` shows colored output |
| 4.1.5 | Integrate `--encoding` flag | Pass encoding to file I/O operations | `src/config.rs`, `src/repl.rs` | Encoding value available in COPY/SOURCE operations |
| 4.1.6 | Integrate `--cqlversion` flag | Pass CQL version to driver connection | `src/session.rs` | Connection uses specified CQL version |
| 4.1.7 | Integrate `--protocol-version` flag | Pass native protocol version to driver | `src/session.rs` | Connection uses specified protocol version |
| 4.1.8 | Implement DEBUG command | Toggle debug mode at runtime | `src/repl.rs` | `DEBUG ON` enables verbose output, `DEBUG OFF` disables |
| 4.1.9 | Implement UNICODE command | Display Unicode character handling info | `src/repl.rs` | `UNICODE` prints encoding info matching Python cqlsh |
| 4.1.10 | Implement LOGIN command | Re-authenticate with new credentials | `src/repl.rs`, `src/session.rs` | `LOGIN admin password` reconnects with new auth |

### Phase 4.2: DESCRIBE Extensions

| # | Task | Description | Files | Validation |
|---|------|-------------|-------|------------|
| 4.2.1 | DESCRIBE FULL SCHEMA | Output all DDL including system keyspaces | `src/describe.rs` | Output matches `DESCRIBE FULL SCHEMA` in Python cqlsh |
| 4.2.2 | DESCRIBE INDEX \<name\> | Reconstruct CREATE INDEX DDL from system_schema.indexes | `src/describe.rs` | `DESCRIBE INDEX my_idx` shows correct DDL |
| 4.2.3 | DESCRIBE MATERIALIZED VIEW | Reconstruct CREATE MV DDL from system_schema.views | `src/describe.rs` | `DESCRIBE MATERIALIZED VIEW my_mv` shows correct DDL |
| 4.2.4 | DESCRIBE TYPE / TYPES | Reconstruct CREATE TYPE DDL from system_schema.types | `src/describe.rs` | `DESCRIBE TYPE my_udt` shows correct DDL |
| 4.2.5 | DESCRIBE FUNCTION / FUNCTIONS | Reconstruct CREATE FUNCTION DDL from system_schema.functions | `src/describe.rs` | `DESCRIBE FUNCTION my_func` shows correct DDL |
| 4.2.6 | DESCRIBE AGGREGATE / AGGREGATES | Reconstruct CREATE AGGREGATE DDL from system_schema.aggregates | `src/describe.rs` | `DESCRIBE AGGREGATE my_agg` shows correct DDL |

### Phase 4.3: COPY TO

| # | Task | Description | Files | Validation |
|---|------|-------------|-------|------------|
| 4.3.1 | COPY TO basic | Parse COPY statement, SELECT all rows, write CSV file | `src/copy.rs`, `src/repl.rs` | `COPY ks.table TO '/tmp/out.csv'` produces valid CSV |
| 4.3.2 | COPY TO format options | DELIMITER, QUOTE, ESCAPE, HEADER, NULL, DATETIMEFORMAT, ENCODING | `src/copy.rs` | Each option changes CSV output as expected |
| 4.3.3 | COPY TO numeric options | FLOATPRECISION, DOUBLEPRECISION, DECIMALSEP, THOUSANDSSEP, BOOLSTYLE | `src/copy.rs` | Numeric formatting matches Python cqlsh |
| 4.3.4 | COPY TO pagination | PAGESIZE, PAGETIMEOUT options for fetch control | `src/copy.rs` | Large table exports correctly with pagination |
| 4.3.5 | COPY TO token ranges | BEGINTOKEN, ENDTOKEN for parallel data extraction | `src/copy.rs` | Token range export produces correct subset |
| 4.3.6 | COPY TO concurrency | MAXREQUESTS concurrent page fetches via Tokio tasks | `src/copy.rs` | Multiple concurrent requests visible in trace |
| 4.3.7 | COPY TO limits & progress | MAXOUTPUTSIZE, REPORTFREQUENCY | `src/copy.rs` | Row limit enforced; progress printed to stderr |
| 4.3.8 | COPY TO stdout | Support writing to stdout instead of file | `src/copy.rs` | `COPY ks.table TO STDOUT` writes to stdout |

### Phase 4.4: COPY FROM

| # | Task | Description | Files | Validation |
|---|------|-------------|-------|------------|
| 4.4.1 | COPY FROM basic | Parse COPY statement, read CSV, INSERT rows | `src/copy.rs` | `COPY ks.table FROM '/tmp/in.csv'` imports data |
| 4.4.2 | COPY FROM format options | All shared format options (DELIMITER, QUOTE, ESCAPE, HEADER, NULL, etc.) | `src/copy.rs` | CSV with custom format imports correctly |
| 4.4.3 | COPY FROM batching | CHUNKSIZE, MINBATCHSIZE, MAXBATCHSIZE | `src/copy.rs` | Rows batched correctly per config |
| 4.4.4 | COPY FROM prepared statements | PREPAREDSTATEMENTS option (prepared vs unprepared INSERT) | `src/copy.rs` | Toggle between prepared and unprepared inserts |
| 4.4.5 | COPY FROM TTL | TTL option on inserted rows | `src/copy.rs` | Inserted rows have correct TTL |
| 4.4.6 | COPY FROM parallelism | NUMPROCESSES parallel Tokio workers | `src/copy.rs` | Parallel workers visible in throughput increase |
| 4.4.7 | COPY FROM rate limiting | INGESTRATE token bucket implementation | `src/copy.rs` | Throughput limited to specified rate |
| 4.4.8 | COPY FROM retry logic | MAXATTEMPTS retry on failed batches | `src/copy.rs` | Failed batches retried before giving up |
| 4.4.9 | COPY FROM error handling | MAXPARSEERRORS, MAXINSERTERRORS, ERRFILE | `src/copy.rs` | Errors counted, written to file, abort on threshold |
| 4.4.10 | COPY FROM progress & stdin | REPORTFREQUENCY, stdin input mode | `src/copy.rs` | Progress to stderr; `COPY ks.table FROM STDIN` works |

---

## Dependencies

| ID | Dependency | Required By |
|----|-----------|-------------|
| DEP-01 | `csv` crate | COPY TO/FROM (4.3, 4.4) |
| DEP-02 | Phase 2 driver & session | All tasks |
| DEP-03 | Phase 3 colorizer & formatter | Output coloring in batch mode |
| DEP-04 | `system_schema.*` table access | DESCRIBE extensions (4.2) |

## Testing Strategy

| ID | Test Type | Description | Validation |
|----|-----------|-------------|------------|
| TEST-01 | Unit | COPY TO CSV formatting with all options | `cargo test copy::tests` passes |
| TEST-02 | Unit | COPY FROM CSV parsing with all options | `cargo test copy::tests` passes |
| TEST-03 | Integration | COPY TO round-trip (export then import) | Data matches after round-trip |
| TEST-04 | Integration | COPY FROM with 10K+ rows | All rows inserted correctly |
| TEST-05 | Integration | COPY error handling (bad CSV, connection errors) | Errors counted and logged to ERRFILE |
| TEST-06 | Unit | DESCRIBE INDEX/VIEW/TYPE/FUNCTION/AGGREGATE DDL | Output matches Python cqlsh |
| TEST-07 | Integration | `-e` and `-f` batch execution | Exit codes correct |
| TEST-08 | Integration | LOGIN re-authentication | Session uses new credentials |

## Risks

| ID | Risk | Mitigation |
|----|------|-----------|
| RISK-01 | COPY FROM performance may not match Python multiprocessing | Tokio tasks are lightweight; benchmark early |
| RISK-02 | CSV edge cases (embedded newlines, quotes) | Use `csv` crate which handles RFC 4180 |
| RISK-03 | Token range calculation complexity | Study Python implementation closely |
| RISK-04 | DESCRIBE DDL reconstruction may miss edge cases | Test against real schemas with UDTs, frozen types, etc. |

## Open Questions

| # | Question | Status | Decision |
|---|----------|--------|----------|
| 1 | Should COPY use CQL BATCH or individual statements? | Resolved | Individual prepared statements (matches Python cqlsh) |
| 2 | Max column width for COPY TO output? | Open | Follow Python cqlsh defaults |
| 3 | Safe mode (`--safe-mode`) in scope for Phase 4? | Open | Consider as stretch goal |
| 4 | `--secure-connect-bundle` (Astra DB) in scope? | Open | Depends on scylla-rs support |

---

## Suggested Implementation Order

```
4.1 (Non-interactive mode + CLI flags)  ← Foundation, unblocks testing
  ↓
4.2 (DESCRIBE extensions)               ← Independent, medium complexity
  ↓
4.3 (COPY TO)                           ← Core feature, builds CSV infra
  ↓
4.4 (COPY FROM)                         ← Reuses CSV infra from COPY TO
```

## Exit Criteria

- [ ] All 24 CLI flags functional
- [ ] `-e` and `-f` batch execution with exit codes
- [ ] LOGIN, DEBUG, UNICODE commands working
- [ ] DESCRIBE covers all 6 schema object types
- [ ] COPY TO with all 17 options
- [ ] COPY FROM with all 19 options
- [ ] Large dataset test (1M+ rows COPY round-trip)
- [ ] Progress reporting matches Python cqlsh format

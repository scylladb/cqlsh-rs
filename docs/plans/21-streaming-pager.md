# Sub-Plan SP21: Streaming Pager (Lazy Page Fetching)

> Parent: [high-level-design.md](high-level-design.md) | Phase 3 (Output & Formatting)
>
> **This is a living document.** Update it as development progresses.

## Objective

Implement lazy/streaming result delivery so that query results are fetched page-by-page from the database on demand as the user scrolls in the pager, rather than loading the entire result set into memory before display.

---

## Problem Statement

Currently, the pipeline is:

```
DB → execute_unpaged/drain all rows → Vec<CqlRow> → format entire table → String → Cursor → pager
```

This means a `SELECT * FROM large_table` with millions of rows will:
1. Fetch all rows from the server into memory
2. Format the entire table into a single string
3. Pass the string to the pager

This is unacceptable for large result sets. The correct behavior (matching Python cqlsh) is to fetch pages incrementally as the user scrolls.

## Requirements & Constraints

| ID | Type | Description |
|----|------|-------------|
| REQ-01 | Requirement | Default SELECT queries use server-side paging (page_size from config or default 100) |
| REQ-02 | Requirement | Rows are fetched from the server only when the pager needs more content to display |
| REQ-03 | Requirement | Memory usage is bounded — only buffered pages + rendered output in the pager |
| REQ-04 | Requirement | The pager displays rows incrementally as they arrive (no waiting for full result) |
| REQ-05 | Requirement | Non-paged mode (piped output, `--no-pager`) still works and streams to stdout |
| CON-01 | Constraint | Must not break existing formatting (tabular, expanded, colorized) |
| CON-02 | Constraint | streampager supports streaming via `add_stream()` with `impl Read` — use this |
| CON-03 | Constraint | Python cqlsh uses `page_size = 100` by default via `default_fetch_size` in cqlshrc |
| GUD-01 | Guideline | Row count footer (`(N rows)`) can be deferred until stream completes |
| GUD-02 | Guideline | Column widths: use first-page heuristic, then fixed — don't reformat on later pages |

## Design Decisions

| Decision | Choice | Rationale | Alternatives Rejected |
|----------|--------|-----------|----------------------|
| Streaming mechanism | `os_pipe` connecting async writer task → pager's `add_stream` | Simple, no custom Read impl, works with streampager | Channel-based `impl Read` (more complex), temp file (disk I/O) |
| Column width strategy | Fix column widths after first page | Avoids reformat jitter; matches Python cqlsh behavior | Dynamic reformat (expensive, visually unstable) |
| Row stream type | Return `RowStream` (async stream) from driver | Preserves lazy fetching semantics from scylla driver | Collected `Vec` (current, defeats purpose) |
| Page size source | `cqlshrc [connection] default_fetch_size`, default 100 | Matches Python cqlsh configuration | Hardcoded (not configurable), CLI flag only |
| Table header | Rendered from first page columns, written once at stream start | User sees column names immediately | Deferred (confusing UX) |

## Implementation Tasks

### Phase 21.1: Driver — Expose Row Stream ✅

| # | Task | Description | Validation | Status |
|---|------|-------------|------------|--------|
| 21.1.1 | Add `RowStream` type to `driver/types.rs` | Define `CqlRowStream` with columns + pinned boxed stream | Compiles | ✅ |
| 21.1.2 | Add `execute_streaming` to `CqlDriver` trait | `async fn execute_streaming(&self, query: &str, page_size: i32) -> Result<CqlRowStream>` | Trait compiles | ✅ |
| 21.1.3 | Implement `execute_streaming` for `ScyllaDriver` | Uses `query_iter` + `rows_stream`, returns stream without draining | Compiles, clippy clean | ✅ |
| 21.1.4 | Add `execute_streaming` to `CqlSession` | Pass through to driver | Compiles | ✅ |

### Phase 21.2: Streaming Formatter ✅

| # | Task | Description | Validation | Status |
|---|------|-------------|------------|--------|
| 21.2.1 | Add `StreamingTableFormatter` in `formatter.rs` | Takes column metadata + `&mut dyn Write`, computes widths from first page, then streams | Compiles, clippy clean | ✅ |
| 21.2.2 | First-page buffering for column widths | Buffers first page, computes column widths, writes header + buffered rows, then streams | Compiles | ✅ |
| 21.2.3 | Row count tracking | `finish()` writes footer `(N rows)` | Compiles | ✅ |
| 21.2.4 | Support expanded format streaming | Expanded format streams directly without width pre-computation | Compiles | ✅ |

### Phase 21.3: Pipe + Pager Integration ✅

| # | Task | Description | Validation | Status |
|---|------|-------------|------------|--------|
| 21.3.1 | Add `os_pipe` dependency | Added `os_pipe = "1"` to `Cargo.toml` | Compiles | ✅ |
| 21.3.2 | Create `page_stream` function in `pager.rs` | Returns `PipeWriter`, spawns pager on background thread reading from pipe | Compiles | ✅ |
| 21.3.3 | Wire streaming pipeline in REPL | SELECT queries use `execute_streaming` → `StreamingTableFormatter` → `PipeWriter` → pager | Compiles, 472 tests pass | ✅ |
| 21.3.4 | Graceful cancellation | Broken pipe from pager quit stops the row stream via drop | By design (stream dropped on write error) | ✅ |
| 21.3.5 | Non-pager fallback | Falls back to existing non-streaming path when pager disabled or not TTY | Existing path unchanged | ✅ |

### Phase 21.4: Configuration & Compatibility ✅

| # | Task | Description | Validation | Status |
|---|------|-------------|------------|--------|
| 21.4.1 | Parse `default_fetch_size` from cqlshrc `[connection]` section | Added to `ConnectionSection` and `MergedConfig`, default 100 | Compiles, clippy clean | ✅ |
| 21.4.2 | Use configured page_size in REPL execute path | `config.fetch_size` passed to `execute_streaming` | Compiles | ✅ |
| 21.4.3 | `PAGING ON/OFF` shell command | Existing `paging_enabled` boolean gates streaming vs non-streaming path | Already works | ✅ |

## Dependencies

| ID | Dependency | Required By |
|----|-----------|-------------|
| DEP-01 | `os_pipe` crate | Phase 21.3 |
| DEP-02 | `futures::Stream` (already via scylla driver) | Phase 21.1 |
| DEP-03 | Existing `streampager` integration | Phase 21.3 |

## Testing Strategy

| ID | Test Type | Description | Validation |
|----|-----------|-------------|------------|
| TEST-01 | Unit | `RowStream` yields rows correctly from mock data | `cargo test` |
| TEST-02 | Unit | `StreamingTableFormatter` output matches `print_tabular` for identical data | Snapshot test with `insta` |
| TEST-03 | Integration | Stream 10,000 rows from testcontainer, verify memory stays bounded | RSS check or row-count assertion without OOM |
| TEST-04 | Integration | `PAGING OFF` + large query still works (unpaged fallback) | assert_cmd test |
| TEST-05 | Manual | Quit pager mid-stream, verify clean exit | No panic, no error on stderr |
| TEST-06 | Integration | Piped output streams without buffering entire result | `cqlsh-rs -e "SELECT..." | head -5` exits quickly |

## Risks

| ID | Risk | Mitigation |
|----|------|-----------|
| RISK-01 | Column width miscalculation on first page (later rows wider) | Truncate or wrap cells that exceed computed width; document behavior |
| RISK-02 | streampager doesn't handle pipe correctly on all platforms | Fall back to current buffered mode if pipe setup fails |
| RISK-03 | Broken pipe handling differs across OS | Catch `BrokenPipe` in writer task, signal cancellation cleanly |
| RISK-04 | `CTRL+C` during streaming may leave driver connection in bad state | Use `query_iter`'s drop semantics (driver handles cancellation) |

## Open Questions

| # | Question | Status | Decision |
|---|----------|--------|----------|
| 1 | Should we support `LIMIT` detection to skip streaming for small results? | Open | — |
| 2 | Should column widths adapt if all first-page values are NULL/short? | Open | — |
| 3 | Does Python cqlsh re-render on terminal resize during paging? | Open | Likely no — investigate |

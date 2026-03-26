# Sub-Plan SP6: Output Formatting

> Parent: [high-level-design.md](high-level-design.md) | Phase: 2-3
> **Status: COMPLETED** — All 20 implementation steps done (2026-03-22). Tabular, expanded, color, type-aware formatting all implemented.

## Objective

Implement all output formatting modes (tabular, JSON, CSV, expanded) with type-aware value rendering, pagination, syntax highlighting, and color support matching Python cqlsh output exactly.

---

## Research Phase

### Tasks

1. **Python cqlsh output format** — Exact table border characters, alignment, spacing
2. **Type-specific formatting** — How each CQL type is displayed (timestamps, UUIDs, blobs, collections, UDTs, tuples)
3. **Color scheme** — Exact ANSI colors used by Python cqlsh for each element
4. **Pagination behavior** — `--More--` prompt, page size calculation, terminal height detection
5. **JSON output format** — Exact JSON structure for each result row
6. **CSV output format** — RFC 4180 compliance, quoting rules

### Research Deliverables

- [x] Exact table border format specification (character-level)
- [x] CQL type -> display string mapping for all types
- [x] ANSI color code table for all output elements
- [x] JSON output schema
- [x] Pagination algorithm specification

---

## Execution Phase

### Implementation Steps

| Step | Description | Module | Tests |
|------|-------------|--------|-------|
| 1 | Basic tabular output (headers + rows + borders) | `formatter.rs` | Snapshot: simple table |
| 2 | Column width calculation (based on header + data) | `formatter.rs` | Unit: width calc |
| 3 | Type-aware value formatting (see `types.rs` plan) | `formatter.rs`, `types.rs` | Unit: each CQL type |
| 4 | NULL value display (empty or configurable) | `formatter.rs` | Unit: null display |
| 5 | Row count footer (`(N rows)`) | `formatter.rs` | Unit: footer format |
| 6 | Query timing display | `formatter.rs` | Unit: timing format |
| 7 | Pagination with `--More--` | `formatter.rs` | Manual: pagination flow |
| 8 | Terminal height detection | `formatter.rs` | Unit: terminal size |
| 9 | Page size configuration (cqlshrc `page_size`) | `formatter.rs` | Unit: config integration |
| 10 | `--no-pager` flag | `formatter.rs` | Unit: disable pagination |
| 11 | Expanded/vertical output mode | `formatter.rs` | Snapshot: expanded format |
| 12 | JSON output format | `formatter.rs` | Snapshot: JSON output |
| 13 | CSV output format | `formatter.rs` | Snapshot: CSV output |
| 14 | Syntax highlighting (CQL keywords) | `colorizer.rs` | Unit: keyword coloring |
| 15 | Result header coloring | `colorizer.rs` | Unit: header colors |
| 16 | Error message coloring (red) | `colorizer.rs` | Unit: error colors |
| 17 | `--color` / `--no-color` flags | `colorizer.rs` | Unit: color toggle |
| 18 | Auto-detect color support (TTY check) | `colorizer.rs` | Unit: TTY detection |
| 19 | Tracing output formatting | `formatter.rs` | Snapshot: trace output |
| 20 | Error display with server error codes | `formatter.rs` | Snapshot: error display |

### Acceptance Criteria

- [x] Tabular output matches Python cqlsh character-for-character (border chars, spacing)
- [x] All CQL types format correctly (see SP9 for type details)
- [x] JSON output is valid JSON and matches Python cqlsh structure
- [x] CSV output is RFC 4180 compliant
- [x] Expanded mode shows each row as key-value pairs
- [x] Pagination pauses at terminal height with `--More--` prompt
- [x] Colors match Python cqlsh defaults
- [x] Color can be forced on/off with flags

---

## Skills Required

- Terminal table rendering (C4: `comfy-table`)
- ANSI color codes (C: `owo-colors`)
- Terminal size detection (`crossterm`)
- JSON serialization (C6: `serde_json`)
- CSV writing (C7: `csv`)
- Snapshot testing (C12: `insta`)

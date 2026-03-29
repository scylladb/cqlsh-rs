# Sub-Plan SP11: Benchmarking

> Parent: [high-level-design.md](high-level-design.md) | Phase: 5
> **Status: IN PROGRESS** — All micro-benchmarks live (startup, parser, format, completion). CI workflow and GitHub Pages dashboard deployed. Python startup baseline measured (200× faster). Remaining: COPY macro-benchmarks, memory benchmarks, and P1–P3 optimisations.

## Objective

Create a comprehensive benchmark suite that measures cqlsh-rs performance across all key dimensions and provides reproducible comparisons against Python cqlsh.

---

## Research Phase

### Tasks

1. **Benchmark tools** — criterion, hyperfine, dhat, heaptrack, flamegraph
2. **Python cqlsh performance baseline** — Measure startup, query, COPY in Python cqlsh
3. **CI benchmark tracking** — github-action-benchmark, bencher.dev, custom solutions
4. **Statistical methodology** — Warmup, iterations, confidence intervals

### Research Deliverables

- [x] Benchmark tool selection rationale — criterion 0.5 for micro-benchmarks
- [x] Python cqlsh baseline measurements — startup 362 ms (hyperfine, CI), cqlsh-rs 1.8 ms → **200× faster**
- [x] CI tracking setup design — benchmark-action/github-action-benchmark@v1 with GitHub Pages
- [x] Benchmark methodology specification — criterion defaults (100 samples, 5s warmup, statistical significance)

---

## Implementation Status

### Implemented

- [x] **Startup micro-benchmarks** — `benches/startup.rs` with criterion 0.5
- [x] **Parser micro-benchmarks** — `benches/parser.rs` with criterion 0.5
- [x] **Formatter micro-benchmarks** — `benches/format.rs` with criterion 0.5
- [x] **Completion micro-benchmarks** — `benches/completion.rs` with criterion 0.5
- [x] **Python internal benchmarks** — `benchmarks/python_cqlsh/benchmarks/bench_internals.py` for parser/format comparison
- [x] **Library crate** — `src/lib.rs` exposes modules for benchmark access
- [x] **CI workflow** — `.github/workflows/bench.yml` with conditional execution
- [x] **GitHub Pages deployment** — Historical dashboard at `https://fruch.github.io/cqlsh-rs/dev/bench/`
- [x] **Artifact collection** — Criterion HTML reports + raw output retained 90 days

### Baseline Results — Startup (SP1)

| Benchmark | Result |
|-----------|--------|
| `cli_parse_args/no_args` | ~14.7 µs |
| `cli_parse_args/full_connection` | ~35.3 µs |
| `cli_validate` | ~2 ns |
| `cqlshrc_parse/empty` | ~2.6 µs |
| `cqlshrc_parse/full` | ~41.7 µs |
| `cqlshrc_parse_scaling/certfiles/100` | ~86 µs |
| `config_merge/all_defaults` | ~217 ns |
| `config_merge/full_merge` | ~1.0 µs |
| `cqlshrc_load_file/full` | ~43 µs |
| `end_to_end_startup/minimal` | ~20 µs |
| `end_to_end_startup/full` | ~95 µs |

> End-to-end startup is well under the 50 ms target (vs Python cqlsh ~800 ms).

### Baseline Results — Statement Parser (SP4)

> Measured on CI: GitHub Actions `ubuntu-latest`, Rust 1.94.1, criterion 0.5 (100 samples, 5 s warmup).

| Benchmark | Result | Notes |
|-----------|--------|-------|
| `parse_statement/simple_select` | 354 ns | floor |
| `parse_statement/simple_insert` | 829 ns | |
| `parse_statement/complex_select` | 1.44 µs | longer token stream |
| `parse_statement/string_literals` | 928 ns | internal semicolons handled |
| `parse_statement/dollar_quoted` | 1.09 µs | `$$` end-marker scan |
| `parse_statement/nested_comments` | 531 ns | |
| `parse_multiline/6_lines` | 3.82 µs | incremental feed_line |
| `parse_multiline/with_comments` | 1.47 µs | |
| `parse_batch/5_statements` | 4.23 µs | 846 ns/stmt |
| `parse_batch/insert_statements/10` | 6.31 µs | 631 ns/stmt |
| `parse_batch/insert_statements/50` | 31.5 µs | 630 ns/stmt ✅ linear |
| `parse_batch/insert_statements/100` | 64.3 µs | 643 ns/stmt ✅ linear |
| `parse_batch/insert_statements/500` | 329.9 µs | 660 ns/stmt ✅ linear |
| `classify_input/empty` | 5.3 ns | |

> **O(n) confirmed:** 10→50→100→500 statements scale 1.0×→5.0×→10.2×→52.3× — perfectly
> linear. Per-statement cost is a stable 630–660 ns across batch sizes.
> Dollar-quoted strings cost ~3× a simple SELECT (end-marker byte scan).
> `classify_input` is effectively free at 5 ns.

### Baseline Results — Output Formatting (SP6 + SP9)

> Measured on CI: GitHub Actions `ubuntu-latest`, Rust 1.94.1, criterion 0.5 (100 samples, 5 s warmup).

| Benchmark | Result | Notes |
|-----------|--------|-------|
| `format_table/rows/10` | 52.1 µs | 5.2 µs/row |
| `format_table/rows/100` | 509.8 µs | 5.1 µs/row ✅ linear |
| `format_table/rows/1000` | 5.06 ms | 5.1 µs/row ✅ linear |
| `format_table_colored/rows/10` | 130.9 µs | 2.5× color overhead |
| `format_table_colored/rows/100` | 1.215 ms | 2.4× color overhead ❌ over 1 ms target |
| `format_expanded/rows/10` | 6.92 µs | 7.5× faster than tabular |
| `format_expanded/rows/100` | 69.1 µs | 7.4× faster than tabular |
| `format_each_type/all_types_tabular` | ~58.4 µs | |
| `format_each_type/all_types_expanded` | ~4.4 µs | |
| `format_edge_cases/empty_result` | ~7.8 ns | effectively free |
| `format_edge_cases/zero_rows` | ~26.4 ns | |
| `format_edge_cases/wide_20col_10rows` | ~256 µs | 1.28 µs/cell for 200 cells |

> **Target met (no color):** Format 100 rows (table) = 510 µs, under the 1 ms target.
> **Target missed (color):** 1.215 ms — color adds 2.4–2.5× overhead via comfy-table ANSI passes.
> Expanded format is ~7.5× faster than tabular (no column-width scan). Scaling is perfectly
> linear: 10→100→1000 rows = 1×→9.8×→97× (stable ~5 µs/row for no-color tabular).

#### CqlValue Display Performance

| Type | `to_string()` Time | Notes |
|------|-------------------|-------|
| `text` | 31.9 ns | |
| `int` | ~54 ns | |
| `bigint` | ~63 ns | |
| `boolean` | ~26 ns | cheapest scalar |
| `double` | 138 ns | Grisu3 float formatting — 5× slower than boolean |
| `uuid` | ~40 ns | |
| `blob` | ~102 ns | hex encoding |
| `null` | ~29 ns | |
| `list<int>` (3 elements) | ~121 ns | |
| `map<text,int>` (2 entries) | 212.7 ns | scales with entry count |

> Individual value formatting is sub-215 ns for all types. The `double` type costs 138 ns
> due to Grisu3 float-to-string conversion — 5× more than `boolean`. Collection types scale
> linearly with element count. These results confirm that the formatting bottleneck is
> comfy-table layout (~500 heap allocations per 100-row table), not value serialization.

### Baseline Results — Tab Completion (SP5)

> Measured on CI: GitHub Actions `ubuntu-latest`, Rust 1.94.1, criterion 0.5 (100 samples, 5 s warmup).

| Benchmark | Result | Notes |
|-----------|--------|-------|
| `complete_keyword/empty_input` | 3.14 µs | all keywords — O(46) linear scan |
| `complete_keyword/prefix_S` | ~3.7 µs | partial match scan |
| `complete_keyword/prefix_SEL` | ~2.5 µs | fewer matches |
| `complete_keyword/prefix_SELECT` | ~2.7 µs | exact match |
| `complete_keyword/clause_after_select` | ~37.4 µs | clause keyword scan — worst case |
| `complete_context/detect/empty` | ~12.3 µs | |
| `complete_context/detect/keyword_start` | ~2.7 µs | |
| `complete_context/detect/after_from` | ~1.0 µs | |
| `complete_context/detect/consistency` | ~4.1 µs | |
| `complete_context/detect/describe` | ~4.8 µs | |
| `complete_context/detect/use_keyspace` | ~693 ns | |
| `complete_context/detect/source_file` | 19.49 µs | file-path prefix scan |
| `complete_context/detect/where_clause` | ~1.3 µs | |
| `complete_consistency/all_levels` | ~3.6 µs | |
| `complete_consistency/prefix_L` | ~1.1 µs | |
| `complete_consistency/serial` | ~2.8 µs | |
| `complete_describe/sub_commands` | ~3.4 µs | |
| `complete_describe/prefix_K` | ~864 ns | |
| `complete_describe/desc_shorthand` | ~2.9 µs | |

> **Target met:** All completion operations complete in <50 µs, far under the 50 ms target.
> Worst case is `clause_after_select` at ~37 µs — this scans all clause keywords via O(n)
> linear search. `source_file` at 19.5 µs allocates during file-path prefix expansion.
> These measurements use an empty schema cache (keyword-only). Schema-backed completions
> (table/column names) will be measured once SP2 (driver & connection) is implemented.

---

## Performance Analysis & Optimization Roadmap

> Measured: GitHub Actions ubuntu-latest, Rust 1.94.1, criterion 0.5. Python comparison via hyperfine.

### Summary vs Targets

| Metric | Target | Actual | vs Python cqlsh | Status |
|--------|--------|--------|-----------------|--------|
| Cold startup | <50 ms | **1.8 ms** | **200× faster** | ✅ |
| Format 100 rows (no color) | <1 ms | 510 µs | ~20× faster (est.) | ✅ |
| Format 100 rows (color) | <1 ms | **1.215 ms** | ~8× faster (est.) | ❌ needs P1 |
| Tab completion (keyword) | <50 ms | <40 µs | — | ✅ |
| Parser O(n) scaling | confirmed | 630–660 ns/stmt | — | ✅ |
| Parser simple SELECT | — | 354 ns | — | ✅ |
| Double formatting | — | 138 ns | — | ℹ️ P2 opportunity |

### Proposed Optimisations

**P1 — Fix colored-table overhead (color: 1.215 ms → target <0.8 ms)**
The 2.4–2.5× color overhead comes from comfy-table re-scanning every cell to apply ANSI
escapes after layout. Pre-colorizing values *before* passing them to comfy-table avoids the
second pass. Alternatively, bypass comfy-table entirely for large result sets and emit
pre-computed column-width ASCII tables directly (see P5).
> Effort: Medium | Impact: High | Affects: every query with color enabled

**P2 — `ryu` for double/float formatting (138 ns → ~35 ns)**
`double` costs 138 ns per value — 5× more than `boolean` — due to Rust's default
Grisu3 float-to-string path. The `ryu` crate (Ryū algorithm) produces correct shortest
representations in ~35 ns with no heap allocation. Drop-in replacement in `CqlValue::fmt`.
> Effort: Low | Impact: Medium | Affects: any result set with float/double columns

**P3 — `phf` or prefix-trie for keyword/clause lookup (O(46) → O(1) or O(prefix))**
`clause_after_select` at ~37 µs and `complete_keyword/empty_input` at 3.14 µs both
use O(n) linear scans over a static keyword list. A `phf::Set` (perfect hash, zero
runtime cost) gives O(1) exact lookup. A prefix-trie (e.g., `fst` crate) gives O(prefix)
prefix-match completions, collapsing `complete_keyword` to <0.5 µs.
> Effort: Medium | Impact: Medium | Affects: completion latency under heavy typing

**P4 — Cache file-path completions (`source_file` 19.5 µs → <1 µs for repeat calls)**
`complete_context/detect/source_file` at 19.5 µs allocates during directory listing for
every keypress. Cache the directory listing with a 100 ms TTL keyed on the directory
prefix — repeated keypresses in the same directory become a cache lookup (~100 ns).
> Effort: Low | Impact: Low | Affects: SOURCE command file-path completion UX

**P5 — Bypass comfy-table for large result sets (>500 rows)**
For large result sets, comfy-table's full column-width scan is O(rows × cols). A streaming
formatter that pre-computes column widths in a single pass and emits rows directly into a
`BufWriter` would eliminate ~500 heap allocations per 100 rows and scale better for
10K+ row result sets (expected in COPY benchmarks).
> Effort: High | Impact: High | Affects: COPY output, large SELECT results

---

## Execution Phase

### Benchmark Suite

#### Micro-benchmarks (criterion)

**Location:** `benches/`

##### Implemented — `startup.rs`

| Benchmark Group | Benchmarks | What it Measures |
|-----------------|------------|-----------------|
| `cli_parse_args` | `no_args`, `host_only`, `host_and_port`, `execute_mode`, `file_mode`, `full_connection` | CLI argument parsing across argument counts |
| `cli_validate` | `valid_full`, `valid_minimal` | Validation logic speed |
| `cqlshrc_parse` | `empty`, `minimal`, `full` | INI config parsing at varying sizes |
| `cqlshrc_parse_scaling` | `certfiles/0`, `certfiles/10`, `certfiles/50`, `certfiles/100` | Config parsing scaling with variable-length sections |
| `config_merge` | `all_defaults`, `cli_overrides_only`, `full_merge` | Four-layer merge (CLI > env > cqlshrc > defaults) |
| `cqlshrc_load_file` | `nonexistent_file`, `minimal_file`, `full_file` | File I/O + parsing combined |
| `end_to_end_startup` | `minimal`, `full` | Complete startup path (parse CLI + load config + merge) |

##### Benchmark Readiness by SP — Add Benchmarks Incrementally

> **Key insight:** Benchmarks should NOT wait until Phase 5. Add each benchmark
> group immediately after its corresponding SP is implemented. The CI
> infrastructure (bench.yml, GitHub Pages dashboard) is already in place.

| SP | Component | Benchmarks Unlocked | Benchmark File |
|----|-----------|---------------------|----------------|
| **SP1** ✅ | CLI & Config | `cli_parse_args`, `cqlshrc_parse`, `config_merge`, `end_to_end_startup` | `startup.rs` ✅ |
| **SP4** ✅ | Statement Parser | `parse_statement`, `parse_multiline`, `parse_batch`, `classify_input` | `parser.rs` ✅ |
| **SP6 + SP9** ✅ | Output Formatting + CQL Types | `format_table_{10,100,1000}`, `format_expanded`, `format_each_type`, `cqlvalue_display` | `format.rs` ✅ |
| **SP5** ✅ | Tab Completion | `complete_keyword`, `complete_context`, `complete_consistency`, `complete_describe` | `completion.rs` ✅ |
| **SP2** | Driver & Connection | Macro-benchmarks: connect + query roundtrip (hyperfine) | `macro/` |
| **SP8** | COPY TO/FROM | COPY throughput macro-benchmarks (hyperfine), COPY memory benchmarks | `macro/` |

> **Action:** After completing each SP above, immediately implement its
> corresponding benchmarks before moving to the next SP. This ensures
> performance regressions are caught early and baselines are established
> while the code is fresh.

##### Implemented — `parser.rs`

| Benchmark Group | Benchmarks | What it Measures |
|-----------------|------------|-----------------|
| `parse_statement` | `simple_select`, `simple_insert`, `complex_select`, `string_literals`, `dollar_quoted`, `nested_comments` | Single-statement parsing across input patterns |
| `parse_multiline` | `6_lines`, `with_comments` | Incremental multi-line feed_line parsing |
| `parse_batch` | `5_statements`, `insert_statements/{10,50,100,500}` | Batch parsing scaling with statement count |
| `classify_input` | `shell_command`, `cql_statement`, `empty`, `use_command` | Input classification latency |

##### Implemented — `format.rs`

| Benchmark Group | Benchmarks | What it Measures |
|-----------------|------------|-----------------|
| `format_table` | `rows/10`, `rows/100`, `rows/1000` | Tabular formatting at various result set sizes |
| `format_table_colored` | `rows/10`, `rows/100` | Tabular formatting with ANSI color overhead |
| `format_expanded` | `rows/10`, `rows/100` | Expanded (vertical) formatting |
| `format_each_type` | `all_types_tabular`, `all_types_expanded` | Formatting across all 14 CQL types |
| `cqlvalue_display` | `text`, `int`, `bigint`, `boolean`, `double`, `uuid`, `blob`, `null`, `list`, `map` | Individual CqlValue::Display performance |
| `format_edge_cases` | `empty_result`, `zero_rows`, `wide_20col_10rows` | Edge case formatting |

##### Implemented — `completion.rs`

| Benchmark Group | Benchmarks | What it Measures |
|-----------------|------------|-----------------|
| `complete_keyword` | `empty_input`, `prefix_S`, `prefix_SEL`, `prefix_SELECT`, `clause_after_select` | Keyword completion latency with varying prefix lengths |
| `complete_context` | `detect/{empty,keyword_start,after_from,consistency,describe,use_keyspace,source_file,where_clause}` | Context detection across 8 input patterns |
| `complete_consistency` | `all_levels`, `prefix_L`, `serial` | Consistency level completion |
| `complete_describe` | `sub_commands`, `prefix_K`, `desc_shorthand` | DESCRIBE sub-command completion |

##### Planned — Future phases

| Benchmark | File | What it Measures |
|-----------|------|-----------------|
| `format_json_100` | `format.rs` | Format 100 rows as JSON (when JSON output is implemented) |
| `format_csv_100` | `format.rs` | Format 100 rows as CSV (when CSV output is implemented) |
| `complete_table` | `completion.rs` | Table name completion with 100 tables (requires live DB) |
| `complete_column` | `completion.rs` | Column completion with 50 columns (requires live DB) |

#### Macro-benchmarks (hyperfine)

| Benchmark | Command | Comparison |
|-----------|---------|------------|
| Cold startup | `cqlsh-rs --version` | `cqlsh --version` |
| Connect + query | `cqlsh-rs -e "SELECT now() FROM system.local"` | Same with `cqlsh` |
| File execution | `cqlsh-rs -f benchmark.cql` | Same with `cqlsh` |
| COPY TO 1K rows | `cqlsh-rs -e "COPY table TO '/tmp/out.csv'"` | Same with `cqlsh` |
| COPY TO 100K rows | Same, larger table | Same with `cqlsh` |
| COPY FROM 1K rows | `cqlsh-rs -e "COPY table FROM '/tmp/in.csv'"` | Same with `cqlsh` |

#### Memory benchmarks (dhat / heaptrack)

| Benchmark | Measurement |
|-----------|------------|
| Idle memory | RSS at idle prompt |
| Query memory | Peak RSS during 10K row query |
| COPY memory | Peak RSS during 100K COPY TO |
| Completion memory | RSS with large schema loaded |

### CI Tracking & Historical Benchmark Reports

> **Reference:** Adopted the pattern from [fruch/coodie](https://github.com/fruch/coodie) — automatic historical tracking of benchmark results with GitHub Pages, regression alerts, and conditional execution.

**Implemented in:** `.github/workflows/bench.yml`

#### Execution Strategy

| Trigger | When | Purpose | Status |
|---------|------|---------|--------|
| Main push | Every merge to `main` | Track historical trends + deploy dashboard | **Implemented** |
| PR with `benchmark` label | On-demand | Compare PR performance impact | **Implemented** |
| Weekly schedule | Monday 06:00 UTC | Catch regressions from dependency updates | **Implemented** |
| Manual dispatch | On-demand | Investigate specific scenarios | **Implemented** |

> Benchmarks do **not** run on every PR (too slow, too noisy). Use the `benchmark` label to opt-in per PR.

#### CI Pipeline Architecture

The workflow consists of two jobs:

1. **`benchmark`** — Runs on all triggers:
   - Installs stable Rust toolchain (dtolnay/rust-toolchain)
   - Caches cargo registry + build artifacts for fast reruns
   - Runs `cargo bench -- --output-format bencher` (all bench targets: startup, parser, format, completion)
   - Uploads criterion HTML report as artifact (90-day retention)
   - Uploads raw bencher output as artifact (90-day retention)
   - Pushes results to `gh-pages` branch via `benchmark-action/github-action-benchmark@v1`
   - Posts regression alerts as PR comments (threshold: 150%)

2. **`deploy-pages`** — Runs only on main pushes (after benchmark job):
   - Checks out `gh-pages` branch (contains historical JSON + auto-generated index.html)
   - Deploys to GitHub Pages via `actions/deploy-pages@v4`
   - Publishes the interactive benchmark dashboard

#### Historical Results Storage

| Layer | Storage | Retention | Purpose |
|-------|---------|-----------|---------|
| Criterion HTML reports | GitHub Actions artifacts | 90 days | Detailed per-run analysis with plots |
| Raw bencher output | GitHub Actions artifacts | 90 days | Post-mortem debugging, re-import |
| Historical JSON data | `gh-pages` branch (`dev/bench/`) | Permanent | Long-term trend data for dashboard |
| GitHub Pages dashboard | GitHub Pages deployment | Permanent | Interactive trend visualization |
| PR comments | PR thread | Permanent | Per-PR regression alerts with before/after numbers |

#### GitHub Pages Dashboard

**URL:** `https://fruch.github.io/cqlsh-rs/dev/bench/`

The dashboard is automatically generated by `benchmark-action/github-action-benchmark` and deployed to GitHub Pages on every merge to `main`. It provides:

- **Interactive time-series charts** — One chart per benchmark group showing performance over time
- **Commit-linked data points** — Each data point links to the commit that produced it
- **Automatic regression detection** — Visual markers when performance degrades
- **Historical comparison** — Compare any two points in the history

**Setup requirements** (one-time, repository settings):
1. Enable GitHub Pages in repository Settings > Pages
2. Set source to "GitHub Actions" (not "Deploy from a branch")
3. The `gh-pages` branch is created automatically on the first benchmark run

#### Comparative Benchmarking

Following the coodie pattern, benchmark against a baseline implementation:

| Variant | Purpose |
|---------|---------|
| **cqlsh-rs** | Project under test |
| **Python cqlsh** | Compatibility & performance target |
| **Raw scylla driver** | Performance floor (minimum possible overhead) |

This allows statements like "cqlsh-rs adds 1.1x overhead vs raw driver" and "cqlsh-rs is 5x faster than Python cqlsh" — both meaningful numbers.

#### Viewing Historical Results

- **Dashboard:** `https://fruch.github.io/cqlsh-rs/dev/bench/` — interactive trend charts (auto-deployed)
- **Criterion reports:** Download HTML artifacts from any workflow run for detailed statistical analysis
- **Raw data:** Download bencher output artifacts for custom analysis or re-import
- **PR comments:** Automatic regression alerts with before/after numbers when `benchmark` label is used

### Benchmark Result Presentation

Three layers provide benchmark visibility at different levels:

1. **Job Summary** — `$GITHUB_STEP_SUMMARY` with grouped Markdown tables (per-run).
   `scripts/format_bench_summary.py` parses bencher output into key results + grouped tables with human-friendly units. Posted automatically on every CI benchmark run.

2. **Hyperfine Comparison** — Rust vs Python side-by-side (per-run).
   `scripts/bench_comparison.sh` runs `hyperfine` comparing `cqlsh-rs --version` vs `cqlsh --version`. Produces a Markdown table with "X times faster" appended to the job summary.

3. **GitHub Pages Dashboard** — Interactive commit-over-commit charts at `https://fruch.github.io/cqlsh-rs/dev/bench/` (historical).
   Generated by `benchmark-action/github-action-benchmark`, deployed automatically on every merge to main. Provides Chart.js line graphs with clickable commit links and tooltips — similar to [coodie's dashboard](https://fruch.github.io/coodie/benchmarks/scylla/).

### Performance Targets

| Metric | Target | Python cqlsh Baseline |
|--------|--------|--------------------|
| Cold startup | <50ms | ~800ms (Python interpreter) |
| Warm startup | <10ms | ~500ms |
| Simple query roundtrip | <5ms overhead | ~50ms overhead |
| Format 100 rows (table) | <1ms | ~10ms |
| Format 100 rows (JSON) | <0.5ms | ~5ms |
| COPY TO throughput | >50K rows/sec | ~20K rows/sec |
| COPY FROM throughput | >30K rows/sec | ~15K rows/sec |
| Tab completion | <50ms | ~100ms |
| Idle RSS | <10MB | ~30MB |
| Binary size | <20MB | N/A (requires Python) |

### Acceptance Criteria

- [x] Startup micro-benchmarks run with statistical significance (criterion)
- [x] All micro-benchmarks run with statistical significance (format, parser, completion)
- [x] Macro-benchmarks show >2x improvement over Python cqlsh in startup — **200× confirmed** (1.8 ms vs 362 ms, hyperfine CI)
- [ ] COPY performance is comparable or better than Python cqlsh
- [ ] Memory usage is lower than Python cqlsh
- [x] CI tracks benchmarks and alerts on regressions (>50% threshold)
- [x] GitHub Pages dashboard deployed with historical trend charts
- [x] Benchmark results are reproducible (criterion 100-sample methodology)
- [x] Artifacts collected and retained (criterion HTML + raw output, 90 days)
- [x] Job summary with grouped Markdown tables posted to `$GITHUB_STEP_SUMMARY`
- [x] Hyperfine Rust-vs-Python startup comparison in CI

---

## Skills Required

- `criterion` benchmarking (C8)
- `hyperfine` CLI benchmarking (S10)
- `dhat` / `heaptrack` memory profiling (S10)
- Flamegraph generation (S10)
- CI/CD with GitHub Actions (S11)
- Statistical methodology for benchmarking (S10)

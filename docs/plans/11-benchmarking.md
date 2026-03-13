# Sub-Plan SP11: Benchmarking

> Parent: [high-level-design.md](high-level-design.md) | Phase: 5

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
- [ ] Python cqlsh baseline measurements
- [x] CI tracking setup design — benchmark-action/github-action-benchmark@v1 with GitHub Pages
- [x] Benchmark methodology specification — criterion defaults (100 samples, 5s warmup, statistical significance)

---

## Implementation Status

### Implemented

- [x] **Startup micro-benchmarks** — `benches/startup.rs` with criterion 0.5
- [x] **Library crate** — `src/lib.rs` exposes modules for benchmark access
- [x] **CI workflow** — `.github/workflows/bench.yml` with conditional execution
- [x] **GitHub Pages deployment** — Historical dashboard at `https://fruch.github.io/cqlsh-rs/dev/bench/`
- [x] **Artifact collection** — Criterion HTML reports + raw output retained 90 days

### Baseline Results (initial measurements)

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
| **SP4** | Statement Parser | `parse_statement`, `parse_multiline` | `parser.rs` |
| **SP2** | Driver & Connection | Macro-benchmarks: connect + query roundtrip (hyperfine) | `macro/` |
| **SP6 + SP9** | Output Formatting + CQL Types | `format_table_{10,100,1000}`, `format_json_100`, `format_csv_100`, `format_each_type` | `format.rs` |
| **SP5** | Tab Completion | `complete_keyword`, `complete_table`, `complete_column` | `completion.rs` |
| **SP8** | COPY TO/FROM | COPY throughput macro-benchmarks (hyperfine), COPY memory benchmarks | `macro/` |

> **Action:** After completing each SP above, immediately implement its
> corresponding benchmarks before moving to the next SP. This ensures
> performance regressions are caught early and baselines are established
> while the code is fresh.

##### Planned — Future phases

| Benchmark | File | What it Measures |
|-----------|------|-----------------|
| `format_table_10` | `format.rs` | Format 10-row result as table |
| `format_table_100` | `format.rs` | Format 100-row result as table |
| `format_table_1000` | `format.rs` | Format 1000-row result as table |
| `format_json_100` | `format.rs` | Format 100 rows as JSON |
| `format_csv_100` | `format.rs` | Format 100 rows as CSV |
| `format_each_type` | `format.rs` | Format each CQL type individually |
| `parse_statement` | `parser.rs` | Parse single CQL statement |
| `parse_multiline` | `parser.rs` | Parse multi-line CQL statement |
| `complete_keyword` | `completion.rs` | Keyword completion latency |
| `complete_table` | `completion.rs` | Table name completion with 100 tables |
| `complete_column` | `completion.rs` | Column completion with 50 columns |

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
   - Runs `cargo bench --bench startup -- --output-format bencher`
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
- [ ] All micro-benchmarks run with statistical significance (format, parser, completion)
- [ ] Macro-benchmarks show >2x improvement over Python cqlsh in startup
- [ ] COPY performance is comparable or better than Python cqlsh
- [ ] Memory usage is lower than Python cqlsh
- [x] CI tracks benchmarks and alerts on regressions (>50% threshold)
- [x] GitHub Pages dashboard deployed with historical trend charts
- [x] Benchmark results are reproducible (criterion 100-sample methodology)
- [x] Artifacts collected and retained (criterion HTML + raw output, 90 days)

---

## Skills Required

- `criterion` benchmarking (C8)
- `hyperfine` CLI benchmarking (S10)
- `dhat` / `heaptrack` memory profiling (S10)
- Flamegraph generation (S10)
- CI/CD with GitHub Actions (S11)
- Statistical methodology for benchmarking (S10)

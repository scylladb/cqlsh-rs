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

- [ ] Benchmark tool selection rationale
- [ ] Python cqlsh baseline measurements
- [ ] CI tracking setup design
- [ ] Benchmark methodology specification

---

## Execution Phase

### Benchmark Suite

#### Micro-benchmarks (criterion)

**Location:** `benches/`

| Benchmark | File | What it Measures |
|-----------|------|-----------------|
| `startup_parse_args` | `startup.rs` | CLI argument parsing speed |
| `startup_load_config` | `startup.rs` | cqlshrc loading and merging |
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

> **Reference:** Adopt the pattern from [fruch/coodie](https://github.com/fruch/coodie) — automatic historical tracking of benchmark results with GitHub Pages, regression alerts, and conditional execution.

#### Execution Strategy

| Trigger | When | Purpose |
|---------|------|---------|
| Main push | Every merge to `main` | Track historical trends |
| PR with `benchmark` label | On-demand | Compare PR performance impact |
| Weekly schedule | Monday 06:00 UTC | Catch regressions from dependency updates |
| Manual dispatch | On-demand | Investigate specific scenarios |

> Benchmarks do **not** run on every PR (too slow, too noisy). Use the `benchmark` label to opt-in per PR.

#### Historical Results Storage

| Layer | Storage | Retention | Purpose |
|-------|---------|-----------|---------|
| JSON artifacts | GitHub Actions artifacts | 90 days | Post-mortem debugging |
| GitHub Pages | `gh-pages` branch | Permanent | Long-term trend visualization |
| PR comments | PR thread | Permanent | Per-PR regression alerts |

#### Workflow

```yaml
# .github/workflows/bench.yml
name: Benchmarks
on:
  push:
    branches: [main]
  pull_request:
    types: [labeled]
  schedule:
    - cron: '0 6 * * 1'  # Weekly Monday 06:00 UTC
  workflow_dispatch:

jobs:
  benchmark:
    # Only run on main push, schedule, dispatch, or when "benchmark" label is added
    if: >
      github.event_name == 'push' ||
      github.event_name == 'schedule' ||
      github.event_name == 'workflow_dispatch' ||
      (github.event_name == 'pull_request' && github.event.label.name == 'benchmark')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run benchmarks
        run: cargo bench --bench all -- --output-format bencher | tee output.txt

      - name: Upload benchmark JSON as artifact
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results-${{ github.sha }}
          path: target/criterion/
          retention-days: 90

      - name: Store results & track history
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: output.txt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          # Push results to gh-pages for historical tracking
          auto-push: ${{ github.event_name == 'push' }}
          # Alert if any benchmark regresses >50% from baseline
          alert-threshold: '150%'
          # Post comment on PR when regression detected
          comment-on-alert: true
          # Fail the workflow on severe regression
          fail-on-alert: false
          # Keep historical data on gh-pages branch
          benchmark-data-dir-path: 'dev/bench'
```

#### Comparative Benchmarking

Following the coodie pattern, benchmark against a baseline implementation:

| Variant | Purpose |
|---------|---------|
| **cqlsh-rs** | Project under test |
| **Python cqlsh** | Compatibility & performance target |
| **Raw scylla driver** | Performance floor (minimum possible overhead) |

This allows statements like "cqlsh-rs adds 1.1x overhead vs raw driver" and "cqlsh-rs is 5x faster than Python cqlsh" — both meaningful numbers.

#### Viewing Historical Results

- **Dashboard:** `https://<user>.github.io/cqlsh-rs/dev/bench/` — auto-generated trend charts
- **Artifacts:** Download JSON from any workflow run for detailed analysis
- **PR comments:** Automatic regression alerts with before/after numbers

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

- [ ] All micro-benchmarks run with statistical significance (criterion)
- [ ] Macro-benchmarks show >2x improvement over Python cqlsh in startup
- [ ] COPY performance is comparable or better than Python cqlsh
- [ ] Memory usage is lower than Python cqlsh
- [ ] CI tracks benchmarks and alerts on regressions (>20%)
- [ ] Benchmark results are reproducible

---

## Skills Required

- `criterion` benchmarking (C8)
- `hyperfine` CLI benchmarking (S10)
- `dhat` / `heaptrack` memory profiling (S10)
- Flamegraph generation (S10)
- CI/CD with GitHub Actions (S11)
- Statistical methodology for benchmarking (S10)

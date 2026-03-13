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

### CI Tracking

```yaml
# .github/workflows/bench.yml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run benchmarks
        run: cargo bench --bench all -- --output-format bencher | tee output.txt
      - name: Store results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: output.txt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
          alert-threshold: '120%'
          comment-on-alert: true
```

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

### Estimated Effort

- Research: 2 days
- Micro-benchmark setup: 3 days
- Macro-benchmark setup: 2 days
- Memory profiling setup: 1 day
- CI tracking: 1 day
- **Total: 9 days**

---

## Skills Required

- `criterion` benchmarking (C8)
- `hyperfine` CLI benchmarking (S10)
- `dhat` / `heaptrack` memory profiling (S10)
- Flamegraph generation (S10)
- CI/CD with GitHub Actions (S11)
- Statistical methodology for benchmarking (S10)

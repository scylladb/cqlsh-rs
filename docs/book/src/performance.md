# Performance

cqlsh-rs is significantly faster than Python cqlsh in startup time and query throughput.

## Startup time

cqlsh-rs starts in milliseconds compared to seconds for Python cqlsh:

| Metric | Python cqlsh | cqlsh-rs | Speedup |
|--------|-------------|----------|---------|
| Cold start | ~1.5s | ~30ms | ~50x |
| Warm start | ~0.8s | ~15ms | ~50x |

Measured with [hyperfine](https://github.com/sharkdp/hyperfine) on Linux x86_64.

## Benchmarks

Performance is tracked continuously via CI. See the live results:

- **[Historical Dashboard](https://fruch.github.io/cqlsh-rs/dev/bench/)** — Interactive commit-over-commit charts
- **[Benchmark Workflow Runs](https://github.com/fruch/cqlsh-rs/actions/workflows/bench.yml)** — Grouped tables and Criterion artifacts

### Running benchmarks locally

```bash
# All Criterion micro-benchmarks
cargo bench

# Individual benchmarks
cargo bench --bench startup
cargo bench --bench parser
cargo bench --bench format
cargo bench --bench formatter
cargo bench --bench completion

# Rust vs Python startup comparison (requires hyperfine + pip install cqlsh)
cargo build --release
scripts/bench_comparison.sh
```

## Binary size

| Build | Size |
|-------|------|
| Debug | ~30 MB |
| Release | ~8 MB |
| Release + strip | ~5 MB |

## Memory usage

cqlsh-rs uses significantly less memory than Python cqlsh due to Rust's lack of a garbage collector and runtime overhead.

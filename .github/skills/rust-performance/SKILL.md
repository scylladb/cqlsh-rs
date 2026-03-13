---
name: rust-performance
description: >-
  Optimize Rust code for performance using profiling, benchmarking, and
  known optimization patterns. Use when asked to improve performance,
  reduce allocations, optimize hot paths, write benchmarks, or profile
  code. Covers criterion benchmarks, allocation tracking, and common
  Rust performance patterns.
allowed-tools: Read, Edit, Write, Bash, Grep, Glob
---

# Rust Performance Optimization

Identify and fix performance issues in cqlsh-rs using profiling, benchmarking, and idiomatic optimization patterns.

## Workflow

1. **Measure first** — Profile or benchmark before optimizing
2. **Identify bottleneck** — Find the actual hot path
3. **Apply targeted fix** — Optimize the bottleneck, not everything
4. **Verify improvement** — Benchmark before and after

## Benchmarking with Criterion

The project uses criterion for benchmarks. See `docs/plans/11-benchmarking.md` for the benchmarking plan.

### Creating a benchmark

```rust
// benches/formatting.rs
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn bench_format_row(c: &mut Criterion) {
    let row = create_test_row();

    c.bench_function("format_row_10_columns", |b| {
        b.iter(|| format_row(black_box(&row)))
    });
}

fn bench_format_row_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("row_formatting");

    for size in [1, 10, 50, 100] {
        let row = create_test_row_with_columns(size);
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &row,
            |b, row| b.iter(|| format_row(black_box(row))),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_format_row, bench_format_row_group);
criterion_main!(benches);
```

### Running benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench formatting

# Compare against baseline
cargo bench -- --save-baseline before
# ... make changes ...
cargo bench -- --baseline before
```

## Common Optimization Patterns

### 1. Reduce allocations

```rust
// BAD: allocates on every call
fn format_value(val: &CqlValue) -> String {
    format!("{}", val)
}

// GOOD: write into a reusable buffer
fn format_value(val: &CqlValue, buf: &mut String) {
    use std::fmt::Write;
    write!(buf, "{}", val).unwrap();
}
```

### 2. Pre-allocate collections

```rust
// BAD: grows incrementally
let mut results = Vec::new();
for row in rows {
    results.push(process(row));
}

// GOOD: allocate once
let mut results = Vec::with_capacity(rows.len());
for row in rows {
    results.push(process(row));
}

// BEST: use iterator collect (auto-sizes via size_hint)
let results: Vec<_> = rows.iter().map(process).collect();
```

### 3. Avoid unnecessary String allocation

```rust
// BAD: allocates a String just to compare
if column.name().to_string() == "key" { ... }

// GOOD: compare &str directly
if column.name() == "key" { ... }

// Use Cow for conditional ownership
use std::borrow::Cow;
fn normalize_name(name: &str) -> Cow<'_, str> {
    if name.contains(' ') {
        Cow::Owned(name.replace(' ', "_"))
    } else {
        Cow::Borrowed(name)
    }
}
```

### 4. Use `SmallVec` for small collections

```rust
use smallvec::SmallVec;

// Stack-allocated for ≤8 elements, heap for more
type Columns = SmallVec<[Column; 8]>;
```

### 5. Avoid redundant work in loops

```rust
// BAD: recompiles regex each iteration
for line in lines {
    let re = Regex::new(r"^\s*--").unwrap();
    if re.is_match(line) { ... }
}

// GOOD: compile once
let re = Regex::new(r"^\s*--").unwrap();
for line in lines {
    if re.is_match(line) { ... }
}

// BEST: use lazy_static or std::sync::LazyLock
static COMMENT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*--").unwrap()
});
```

### 6. Efficient string building

```rust
// BAD: repeated allocation and concatenation
let mut output = String::new();
for col in columns {
    output = output + &col.name + " | ";
}

// GOOD: pre-allocate and use write!
use std::fmt::Write;
let mut output = String::with_capacity(columns.len() * 20);
for col in columns {
    write!(output, "{} | ", col.name).unwrap();
}
```

### 7. Use `bytes()` for ASCII processing

```rust
// Slower: iterates over chars (UTF-8 decoding)
let count = text.chars().filter(|c| *c == '\n').count();

// Faster: iterates over raw bytes
let count = text.bytes().filter(|b| *b == b'\n').count();

// Even faster for search: use memchr
let count = memchr::memchr_iter(b'\n', text.as_bytes()).count();
```

### 8. Async performance

```rust
// BAD: sequential async calls
let a = fetch_a().await?;
let b = fetch_b().await?;

// GOOD: concurrent when independent
let (a, b) = tokio::try_join!(fetch_a(), fetch_b())?;

// Avoid holding locks across await points
// BAD
let guard = mutex.lock().await;
do_async_work().await;  // lock held across await!
drop(guard);

// GOOD
let data = {
    let guard = mutex.lock().await;
    guard.clone()
};  // lock released
do_async_work_with(data).await;
```

## Profiling Tools

### Memory profiling

```bash
# Track allocations with DHAT
cargo install dhat
# Add to main.rs: #[global_allocator] static ALLOC: dhat::Alloc = dhat::Alloc;
cargo run --features dhat-heap -- [args]

# Alternatively use jemalloc's profiling
MALLOC_CONF=prof:true,prof_prefix:jeprof cargo run -- [args]
```

### CPU profiling

```bash
# Using perf (Linux)
cargo build --release
perf record --call-graph dwarf ./target/release/cqlsh [args]
perf report

# Using flamegraph
cargo install flamegraph
cargo flamegraph -- [args]
```

### Compile time profiling

```bash
# See what takes long to compile
cargo build --timings
# Opens target/cargo-timings/cargo-timing.html
```

## Performance Checklist

- [ ] Hot paths identified and benchmarked
- [ ] No allocations in tight loops without justification
- [ ] Collections pre-sized when length is known
- [ ] String operations use `&str` parameters where possible
- [ ] Regex compiled once, not per-iteration
- [ ] Async operations use `join!`/`try_join!` for concurrency
- [ ] No blocking I/O on the async runtime
- [ ] `--release` profile used for performance measurements
- [ ] Benchmark results documented for regression tracking

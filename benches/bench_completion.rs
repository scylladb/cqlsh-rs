//! Tab completion benchmarks for cqlsh-rs.
//!
//! Measures the performance of keyword, table, and column completion logic.
//! Since CqlCompleter requires a Tokio runtime and async locks, we benchmark
//! the public-facing complete() method through the completer instance with
//! a pre-populated schema cache.
//!
//! These benchmarks correspond to SP5 in the benchmarking plan.

use criterion::{criterion_group, BenchmarkId, Criterion};
use std::hint::black_box;
use std::sync::Arc;

use rustyline::completion::Completer;
use tokio::sync::RwLock;

use cqlsh_rs::completer::CqlCompleter;
use cqlsh_rs::schema_cache::SchemaCache;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a completer with an empty schema cache (keyword-only completions).
fn make_completer() -> (CqlCompleter, tokio::runtime::Runtime) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = Arc::new(RwLock::new(SchemaCache::new()));
    let current_ks = Arc::new(RwLock::new(None::<String>));
    let completer = CqlCompleter::new(cache, current_ks, rt.handle().clone(), false);
    (completer, rt)
}

/// Perform a completion and return the results.
fn do_complete(
    completer: &CqlCompleter,
    line: &str,
    pos: usize,
) -> Vec<rustyline::completion::Pair> {
    // We need a rustyline Context, but the completer doesn't use it.
    // Create a minimal history for the context.
    let history = rustyline::history::DefaultHistory::new();
    let ctx = rustyline::Context::new(&history);
    let (_start, completions) = completer.complete(line, pos, &ctx).unwrap();
    completions
}

// ---------------------------------------------------------------------------
// Benchmarks: Keyword completion
// ---------------------------------------------------------------------------

fn bench_complete_keyword(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_keyword");
    let (completer, _rt) = make_completer();

    // Complete from empty input (all keywords + shell commands)
    group.bench_function("empty_input", |b| {
        b.iter(|| black_box(do_complete(&completer, black_box(""), 0)))
    });

    // Complete with a short prefix
    group.bench_function("prefix_S", |b| {
        b.iter(|| black_box(do_complete(&completer, black_box("S"), 1)))
    });

    // Complete with a longer prefix (fewer matches)
    group.bench_function("prefix_SEL", |b| {
        b.iter(|| black_box(do_complete(&completer, black_box("SEL"), 3)))
    });

    // Complete with exact match
    group.bench_function("prefix_SELECT", |b| {
        b.iter(|| black_box(do_complete(&completer, black_box("SELECT"), 6)))
    });

    // Complete clause keywords (after a statement keyword)
    group.bench_function("clause_after_select", |b| {
        b.iter(|| black_box(do_complete(&completer, black_box("SELECT * F"), 9)))
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: Context detection speed
// ---------------------------------------------------------------------------

fn bench_complete_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_context");
    let (completer, _rt) = make_completer();

    let inputs: Vec<(&str, &str)> = vec![
        ("empty", ""),
        ("keyword_start", "SEL"),
        ("after_from", "SELECT * FROM "),
        ("consistency", "CONSISTENCY "),
        ("describe", "DESCRIBE "),
        ("use_keyspace", "USE "),
        ("source_file", "SOURCE '/tmp/"),
        ("where_clause", "SELECT * FROM users WHERE "),
    ];

    for (name, input) in &inputs {
        let pos = input.len();
        group.bench_with_input(BenchmarkId::new("detect", *name), input, |b, input| {
            b.iter(|| black_box(do_complete(&completer, black_box(input), pos)))
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: Consistency level completion
// ---------------------------------------------------------------------------

fn bench_complete_consistency(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_consistency");
    let (completer, _rt) = make_completer();

    group.bench_function("all_levels", |b| {
        b.iter(|| black_box(do_complete(&completer, black_box("CONSISTENCY "), 12)))
    });

    group.bench_function("prefix_L", |b| {
        b.iter(|| black_box(do_complete(&completer, black_box("CONSISTENCY L"), 13)))
    });

    group.bench_function("serial", |b| {
        b.iter(|| {
            black_box(do_complete(
                &completer,
                black_box("SERIAL CONSISTENCY "),
                19,
            ))
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: DESCRIBE sub-command completion
// ---------------------------------------------------------------------------

fn bench_complete_describe(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_describe");
    let (completer, _rt) = make_completer();

    group.bench_function("sub_commands", |b| {
        b.iter(|| black_box(do_complete(&completer, black_box("DESCRIBE "), 9)))
    });

    group.bench_function("prefix_K", |b| {
        b.iter(|| black_box(do_complete(&completer, black_box("DESCRIBE K"), 10)))
    });

    group.bench_function("desc_shorthand", |b| {
        b.iter(|| black_box(do_complete(&completer, black_box("DESC "), 5)))
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion harness
// ---------------------------------------------------------------------------

criterion_group!(
    completion_benches,
    bench_complete_keyword,
    bench_complete_context,
    bench_complete_consistency,
    bench_complete_describe,
);

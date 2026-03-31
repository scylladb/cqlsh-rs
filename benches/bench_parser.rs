//! Statement parser benchmarks for cqlsh-rs.
//!
//! Measures the performance of the incremental CQL statement parser across
//! various input patterns: simple statements, multi-line statements, statements
//! with comments, string literals, and batch parsing.
//!
//! These benchmarks correspond to SP4 in the benchmarking plan.

use criterion::{black_box, criterion_group, BenchmarkId, Criterion};

use cqlsh_rs::parser::{classify_input, parse_batch, StatementParser};

// ---------------------------------------------------------------------------
// Sample inputs
// ---------------------------------------------------------------------------

const SIMPLE_SELECT: &str = "SELECT * FROM users WHERE id = 1;";
const SIMPLE_INSERT: &str =
    "INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');";
const COMPLEX_SELECT: &str =
    "SELECT id, name, email, created_at FROM users WHERE id IN (1, 2, 3) AND status = 'active' ORDER BY created_at DESC LIMIT 100;";

const MULTILINE_STATEMENT: &str = "SELECT id, name, email, created_at\n\
     FROM users\n\
     WHERE id IN (1, 2, 3)\n\
     AND status = 'active'\n\
     ORDER BY created_at DESC\n\
     LIMIT 100;";

const STATEMENT_WITH_COMMENTS: &str = "-- Select active users\n\
     SELECT * FROM users /* the main table */\n\
     WHERE status = 'active'; -- only active ones";

const STATEMENT_WITH_STRING_LITERALS: &str =
    "INSERT INTO messages (id, body) VALUES (1, 'Hello; world -- not a comment /* also not */');";

const BATCH_INPUT: &str = "\
-- Schema setup
CREATE TABLE IF NOT EXISTS users (id int PRIMARY KEY, name text, email text);
INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');
INSERT INTO users (id, name, email) VALUES (2, 'Bob', 'bob@example.com');
INSERT INTO users (id, name, email) VALUES (3, 'Charlie', 'charlie@example.com');
SELECT * FROM users;
-- Done
";

const DOLLAR_QUOTED: &str =
    "CREATE FUNCTION ks.my_func() RETURNS NULL ON NULL INPUT RETURNS text LANGUAGE java AS $$return input;$$;";

const NESTED_COMMENTS: &str = "SELECT /* outer /* inner */ still comment */ * FROM users;";

// ---------------------------------------------------------------------------
// Benchmarks: Single statement parsing
// ---------------------------------------------------------------------------

fn bench_parse_statement(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_statement");

    group.bench_function("simple_select", |b| {
        b.iter(|| {
            let mut parser = StatementParser::new();
            black_box(parser.feed_line(black_box(SIMPLE_SELECT)))
        });
    });

    group.bench_function("simple_insert", |b| {
        b.iter(|| {
            let mut parser = StatementParser::new();
            black_box(parser.feed_line(black_box(SIMPLE_INSERT)))
        });
    });

    group.bench_function("complex_select", |b| {
        b.iter(|| {
            let mut parser = StatementParser::new();
            black_box(parser.feed_line(black_box(COMPLEX_SELECT)))
        });
    });

    group.bench_function("string_literals", |b| {
        b.iter(|| {
            let mut parser = StatementParser::new();
            black_box(parser.feed_line(black_box(STATEMENT_WITH_STRING_LITERALS)))
        });
    });

    group.bench_function("dollar_quoted", |b| {
        b.iter(|| {
            let mut parser = StatementParser::new();
            black_box(parser.feed_line(black_box(DOLLAR_QUOTED)))
        });
    });

    group.bench_function("nested_comments", |b| {
        b.iter(|| {
            let mut parser = StatementParser::new();
            black_box(parser.feed_line(black_box(NESTED_COMMENTS)))
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: Multi-line parsing (incremental feed_line)
// ---------------------------------------------------------------------------

fn bench_parse_multiline(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_multiline");

    // Feed the multi-line statement one line at a time (realistic REPL usage)
    let lines: Vec<&str> = MULTILINE_STATEMENT.lines().collect();

    group.bench_function("6_lines", |b| {
        b.iter(|| {
            let mut parser = StatementParser::new();
            let mut result = None;
            for line in &lines {
                result = Some(parser.feed_line(black_box(line)));
            }
            black_box(result)
        });
    });

    // Feed the same statement with comments interspersed
    let commented_lines: Vec<&str> = STATEMENT_WITH_COMMENTS.lines().collect();

    group.bench_function("with_comments", |b| {
        b.iter(|| {
            let mut parser = StatementParser::new();
            let mut result = None;
            for line in &commented_lines {
                result = Some(parser.feed_line(black_box(line)));
            }
            black_box(result)
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: Batch parsing
// ---------------------------------------------------------------------------

fn bench_parse_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_batch");

    group.bench_function("5_statements", |b| {
        b.iter(|| black_box(parse_batch(black_box(BATCH_INPUT))))
    });

    // Scaling benchmark: parse N identical statements
    for count in [10, 50, 100, 500] {
        let mut input = String::new();
        for i in 0..count {
            input.push_str(&format!(
                "INSERT INTO users (id, name) VALUES ({i}, 'user_{i}');\n"
            ));
        }

        group.bench_with_input(
            BenchmarkId::new("insert_statements", count),
            &input,
            |b, input| b.iter(|| black_box(parse_batch(black_box(input)))),
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: Input classification
// ---------------------------------------------------------------------------

fn bench_classify_input(c: &mut Criterion) {
    let mut group = c.benchmark_group("classify_input");

    group.bench_function("shell_command", |b| {
        b.iter(|| black_box(classify_input(black_box("DESCRIBE KEYSPACES"))))
    });

    group.bench_function("cql_statement", |b| {
        b.iter(|| black_box(classify_input(black_box("SELECT * FROM users"))))
    });

    group.bench_function("empty", |b| {
        b.iter(|| black_box(classify_input(black_box(""))))
    });

    group.bench_function("use_command", |b| {
        b.iter(|| black_box(classify_input(black_box("USE my_keyspace"))))
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion harness
// ---------------------------------------------------------------------------

criterion_group!(
    parser_benches,
    bench_parse_statement,
    bench_parse_multiline,
    bench_parse_batch,
    bench_classify_input,
);

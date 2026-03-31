//! Output formatting benchmarks for cqlsh-rs.
//!
//! Measures the performance of tabular, expanded, JSON, and CSV result formatting
//! across various result set sizes and CQL data types.
//!
//! These benchmarks correspond to SP6 + SP9 in the benchmarking plan.

use std::fmt::Write as FmtWrite;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::OnceLock;

use criterion::{criterion_group, BenchmarkId, Criterion};
use std::hint::black_box;

use cqlsh_rs::colorizer::CqlColorizer;
use cqlsh_rs::driver::types::{CqlColumn, CqlResult, CqlRow, CqlValue};
use cqlsh_rs::formatter::{print_expanded, print_tabular};

// ---------------------------------------------------------------------------
// Helpers: Generate synthetic result sets
// ---------------------------------------------------------------------------

/// Create a result set with N rows of typical mixed-type data.
fn make_mixed_result(num_rows: usize) -> CqlResult {
    let columns = vec![
        CqlColumn {
            name: "id".to_string(),
            type_name: "int".to_string(),
        },
        CqlColumn {
            name: "name".to_string(),
            type_name: "text".to_string(),
        },
        CqlColumn {
            name: "email".to_string(),
            type_name: "text".to_string(),
        },
        CqlColumn {
            name: "age".to_string(),
            type_name: "int".to_string(),
        },
        CqlColumn {
            name: "active".to_string(),
            type_name: "boolean".to_string(),
        },
    ];

    let rows: Vec<CqlRow> = (0..num_rows)
        .map(|i| CqlRow {
            values: vec![
                CqlValue::Int(i as i32),
                CqlValue::Text(format!("user_{i}")),
                CqlValue::Text(format!("user_{i}@example.com")),
                CqlValue::Int(20 + (i % 60) as i32),
                CqlValue::Boolean(i % 2 == 0),
            ],
        })
        .collect();

    CqlResult {
        columns,
        rows,
        has_rows: true,
        tracing_id: None,
        warnings: vec![],
    }
}

/// Create a result set exercising every CQL type.
fn make_all_types_result() -> CqlResult {
    let columns = vec![
        CqlColumn {
            name: "ascii_col".to_string(),
            type_name: "ascii".to_string(),
        },
        CqlColumn {
            name: "bigint_col".to_string(),
            type_name: "bigint".to_string(),
        },
        CqlColumn {
            name: "blob_col".to_string(),
            type_name: "blob".to_string(),
        },
        CqlColumn {
            name: "boolean_col".to_string(),
            type_name: "boolean".to_string(),
        },
        CqlColumn {
            name: "double_col".to_string(),
            type_name: "double".to_string(),
        },
        CqlColumn {
            name: "float_col".to_string(),
            type_name: "float".to_string(),
        },
        CqlColumn {
            name: "int_col".to_string(),
            type_name: "int".to_string(),
        },
        CqlColumn {
            name: "text_col".to_string(),
            type_name: "text".to_string(),
        },
        CqlColumn {
            name: "uuid_col".to_string(),
            type_name: "uuid".to_string(),
        },
        CqlColumn {
            name: "inet_col".to_string(),
            type_name: "inet".to_string(),
        },
        CqlColumn {
            name: "list_col".to_string(),
            type_name: "list<int>".to_string(),
        },
        CqlColumn {
            name: "map_col".to_string(),
            type_name: "map<text,int>".to_string(),
        },
        CqlColumn {
            name: "set_col".to_string(),
            type_name: "set<text>".to_string(),
        },
        CqlColumn {
            name: "null_col".to_string(),
            type_name: "text".to_string(),
        },
    ];

    let rows = vec![CqlRow {
        values: vec![
            CqlValue::Ascii("hello".to_string()),
            CqlValue::BigInt(9_223_372_036_854_775_807),
            CqlValue::Blob(vec![0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe]),
            CqlValue::Boolean(true),
            CqlValue::Double(std::f64::consts::PI),
            CqlValue::Float(std::f32::consts::E),
            CqlValue::Int(42),
            CqlValue::Text("The quick brown fox jumps over the lazy dog".to_string()),
            CqlValue::Uuid(uuid::Uuid::nil()),
            CqlValue::Inet(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
            CqlValue::List(vec![CqlValue::Int(1), CqlValue::Int(2), CqlValue::Int(3)]),
            CqlValue::Map(vec![
                (CqlValue::Text("key1".to_string()), CqlValue::Int(100)),
                (CqlValue::Text("key2".to_string()), CqlValue::Int(200)),
            ]),
            CqlValue::Set(vec![
                CqlValue::Text("alpha".to_string()),
                CqlValue::Text("beta".to_string()),
                CqlValue::Text("gamma".to_string()),
            ]),
            CqlValue::Null,
        ],
    }];

    CqlResult {
        columns,
        rows,
        has_rows: true,
        tracing_id: None,
        warnings: vec![],
    }
}

fn make_wide_result(num_cols: usize, num_rows: usize) -> CqlResult {
    let columns: Vec<CqlColumn> = (0..num_cols)
        .map(|i| CqlColumn {
            name: format!("column_{i}"),
            type_name: "text".to_string(),
        })
        .collect();

    let rows: Vec<CqlRow> = (0..num_rows)
        .map(|r| CqlRow {
            values: (0..num_cols)
                .map(|c| CqlValue::Text(format!("row{r}_col{c}_value")))
                .collect(),
        })
        .collect();

    CqlResult {
        columns,
        rows,
        has_rows: true,
        tracing_id: None,
        warnings: vec![],
    }
}

// Pre-built result for JSON/CSV benchmarks — built once, reused across iters.
static RESULT_100: OnceLock<CqlResult> = OnceLock::new();

fn result_100() -> &'static CqlResult {
    RESULT_100.get_or_init(|| make_mixed_result(100))
}

// ---------------------------------------------------------------------------
// Benchmarks: Tabular formatting at various sizes
// ---------------------------------------------------------------------------

fn bench_format_table(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_table");
    let colorizer = CqlColorizer::new(false);

    for num_rows in [10, 100, 1000] {
        let result = make_mixed_result(num_rows);

        group.bench_with_input(BenchmarkId::new("rows", num_rows), &result, |b, result| {
            b.iter(|| {
                let mut buf = Vec::with_capacity(num_rows * 100);
                print_tabular(black_box(result), &colorizer, &mut buf);
                black_box(buf)
            })
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: Tabular formatting with color enabled
// ---------------------------------------------------------------------------

fn bench_format_table_colored(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_table_colored");
    let colorizer = CqlColorizer::new(true);

    for num_rows in [10, 100] {
        let result = make_mixed_result(num_rows);

        group.bench_with_input(BenchmarkId::new("rows", num_rows), &result, |b, result| {
            b.iter(|| {
                let mut buf = Vec::with_capacity(num_rows * 150);
                print_tabular(black_box(result), &colorizer, &mut buf);
                black_box(buf)
            })
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: Expanded (vertical) formatting
// ---------------------------------------------------------------------------

fn bench_format_expanded(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_expanded");
    let colorizer = CqlColorizer::new(false);

    for num_rows in [10, 100] {
        let result = make_mixed_result(num_rows);

        group.bench_with_input(BenchmarkId::new("rows", num_rows), &result, |b, result| {
            b.iter(|| {
                let mut buf = Vec::with_capacity(num_rows * 200);
                print_expanded(black_box(result), &colorizer, &mut buf);
                black_box(buf)
            })
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: Format each CQL type
// ---------------------------------------------------------------------------

fn bench_format_each_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_each_type");
    let colorizer = CqlColorizer::new(false);
    let result = make_all_types_result();

    group.bench_function("all_types_tabular", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(1024);
            print_tabular(black_box(&result), &colorizer, &mut buf);
            black_box(buf)
        })
    });

    group.bench_function("all_types_expanded", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(1024);
            print_expanded(black_box(&result), &colorizer, &mut buf);
            black_box(buf)
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: CqlValue Display (formatting individual values)
// ---------------------------------------------------------------------------

fn bench_cqlvalue_display(c: &mut Criterion) {
    let mut group = c.benchmark_group("cqlvalue_display");

    let values: Vec<(&str, CqlValue)> = vec![
        ("text", CqlValue::Text("Hello, world!".to_string())),
        ("int", CqlValue::Int(42)),
        ("bigint", CqlValue::BigInt(9_223_372_036_854_775_807)),
        ("boolean", CqlValue::Boolean(true)),
        ("double", CqlValue::Double(std::f64::consts::PI)),
        ("uuid", CqlValue::Uuid(uuid::Uuid::nil())),
        ("blob", CqlValue::Blob(vec![0xde, 0xad, 0xbe, 0xef])),
        ("null", CqlValue::Null),
        (
            "list",
            CqlValue::List(vec![CqlValue::Int(1), CqlValue::Int(2), CqlValue::Int(3)]),
        ),
        (
            "map",
            CqlValue::Map(vec![
                (CqlValue::Text("a".to_string()), CqlValue::Int(1)),
                (CqlValue::Text("b".to_string()), CqlValue::Int(2)),
            ]),
        ),
    ];

    for (name, value) in &values {
        group.bench_with_input(BenchmarkId::new("to_string", *name), value, |b, val| {
            b.iter(|| black_box(black_box(val).to_string()))
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: Empty and edge cases
// ---------------------------------------------------------------------------

fn bench_format_edge_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_edge_cases");
    let colorizer = CqlColorizer::new(false);

    // Empty result
    let empty = CqlResult::empty();
    group.bench_function("empty_result", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            print_tabular(black_box(&empty), &colorizer, &mut buf);
            black_box(buf)
        })
    });

    // Zero rows but with columns
    let no_rows = CqlResult {
        columns: vec![
            CqlColumn {
                name: "id".to_string(),
                type_name: "int".to_string(),
            },
            CqlColumn {
                name: "name".to_string(),
                type_name: "text".to_string(),
            },
        ],
        rows: vec![],
        has_rows: true,
        tracing_id: None,
        warnings: vec![],
    };
    group.bench_function("zero_rows", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            print_tabular(black_box(&no_rows), &colorizer, &mut buf);
            black_box(buf)
        })
    });

    // Wide table (20 columns)
    let wide = make_wide_result(20, 10);
    group.bench_function("wide_20col_10rows", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(4096);
            print_tabular(black_box(&wide), &colorizer, &mut buf);
            black_box(buf)
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmarks: JSON serialization baseline
// ---------------------------------------------------------------------------

fn format_json(result: &CqlResult) -> String {
    let col_names: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    let capacity = result.rows.len() * col_names.len() * 20;
    let mut out = String::with_capacity(capacity);

    out.push('[');
    for (ri, row) in result.rows.iter().enumerate() {
        if ri > 0 {
            out.push(',');
        }
        out.push('{');
        for (ci, val) in row.values.iter().enumerate() {
            if ci > 0 {
                out.push(',');
            }
            // key
            write!(out, "\"{}\": ", col_names[ci]).unwrap();
            // value — strings quoted, everything else bare
            match val {
                CqlValue::Ascii(s) | CqlValue::Text(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    write!(out, "\"{escaped}\"").unwrap();
                }
                CqlValue::Null | CqlValue::Unset => out.push_str("null"),
                CqlValue::Boolean(b) => out.push_str(if *b { "true" } else { "false" }),
                other => write!(out, "{other}").unwrap(),
            }
        }
        out.push('}');
    }
    out.push(']');
    out
}

fn bench_format_json_100(c: &mut Criterion) {
    let result = result_100();

    c.bench_function("format_json_100", |b| {
        b.iter(|| format_json(black_box(result)))
    });
}

// ---------------------------------------------------------------------------
// Benchmarks: CSV serialization baseline
// ---------------------------------------------------------------------------

fn format_csv(result: &CqlResult) -> Vec<u8> {
    let mut wtr = csv::Writer::from_writer(Vec::with_capacity(result.rows.len() * 40));

    // Header
    let headers: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    wtr.write_record(&headers).unwrap();

    // Data rows
    for row in &result.rows {
        let record: Vec<String> = row.values.iter().map(|v| v.to_string()).collect();
        wtr.write_record(&record).unwrap();
    }

    wtr.into_inner().unwrap()
}

fn bench_format_csv_100(c: &mut Criterion) {
    let result = result_100();

    c.bench_function("format_csv_100", |b| {
        b.iter(|| format_csv(black_box(result)))
    });
}

// ---------------------------------------------------------------------------
// Criterion harness
// ---------------------------------------------------------------------------

criterion_group!(
    format_benches,
    bench_format_table,
    bench_format_table_colored,
    bench_format_expanded,
    bench_format_each_type,
    bench_cqlvalue_display,
    bench_format_edge_cases,
    bench_format_json_100,
    bench_format_csv_100,
);

//! Formatter micro-benchmarks for cqlsh-rs.
//!
//! Measures output formatting across three groups:
//!
//! - **`format_table/{10,100,1000}`** — tabular output via `print_tabular` at
//!   increasing row counts; establishes the O(n) scaling baseline.
//! - **`format_json_100`** — inline JSON serialisation of 100 rows using
//!   `CqlValue::to_string()` and `std::fmt::Write`; baseline for the real
//!   JSON formatter once SP6+SP9 land.
//! - **`format_csv_100`** — CSV serialisation via the `csv` crate; baseline
//!   for the real CSV formatter.
//! - **`format_each_type`** — one pass of `print_tabular` over a result
//!   containing one value per CQL type; guards against regressions in
//!   per-type Display paths.

use std::fmt::Write as FmtWrite;
use std::net::IpAddr;
use std::sync::OnceLock;

use chrono::{NaiveDate, NaiveTime};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use num_bigint::BigInt;
use uuid::Uuid;

use cqlsh_rs::colorizer::CqlColorizer;
use cqlsh_rs::driver::types::{CqlColumn, CqlResult, CqlRow, CqlValue};
use cqlsh_rs::formatter::print_tabular;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn no_color() -> CqlColorizer {
    CqlColorizer::new(false)
}

/// Build a `CqlResult` with fixed schema and `n` deterministic rows.
///
/// Schema: `(id int, name text, score float, active boolean)`
fn make_result(n: usize) -> CqlResult {
    let columns = vec![
        CqlColumn { name: "id".to_string(),     type_name: "int".to_string() },
        CqlColumn { name: "name".to_string(),   type_name: "text".to_string() },
        CqlColumn { name: "score".to_string(),  type_name: "float".to_string() },
        CqlColumn { name: "active".to_string(), type_name: "boolean".to_string() },
    ];

    let rows = (0..n)
        .map(|i| CqlRow {
            values: vec![
                CqlValue::Int(i as i32),
                CqlValue::Text(format!("user_{i}")),
                CqlValue::Float(i as f32 * 0.5),
                CqlValue::Boolean(i % 2 == 0),
            ],
        })
        .collect();

    CqlResult { columns, rows, has_rows: true, tracing_id: None, warnings: vec![] }
}

// Pre-built results for the table benchmarks — built once, reused across iters.
static RESULT_10:   OnceLock<CqlResult> = OnceLock::new();
static RESULT_100:  OnceLock<CqlResult> = OnceLock::new();
static RESULT_1000: OnceLock<CqlResult> = OnceLock::new();

fn result_10()   -> &'static CqlResult { RESULT_10.get_or_init(||   make_result(10)) }
fn result_100()  -> &'static CqlResult { RESULT_100.get_or_init(||  make_result(100)) }
fn result_1000() -> &'static CqlResult { RESULT_1000.get_or_init(|| make_result(1000)) }

// ---------------------------------------------------------------------------
// format_table/{10,100,1000}
// ---------------------------------------------------------------------------

fn bench_format_table(c: &mut Criterion) {
    let color = no_color();
    let mut group = c.benchmark_group("format_table");

    for (size, result) in [(10, result_10()), (100, result_100()), (1000, result_1000())] {
        group.bench_with_input(BenchmarkId::from_parameter(size), result, |b, r| {
            b.iter(|| {
                let mut buf = Vec::with_capacity(size * 40);
                print_tabular(black_box(r), &color, &mut buf);
                buf
            })
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// format_json_100
//
// Inline JSON serialiser using CqlValue::to_string() and std::fmt::Write.
// Serves as a performance baseline for the real JSON formatter (SP6+SP9).
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
// format_csv_100
//
// CSV serialisation using the `csv` crate (already a project dependency).
// Serves as a performance baseline for the real CSV formatter.
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
// format_each_type
//
// One row containing one value of every CQL type, rendered via print_tabular.
// Guards against per-type Display regressions.
// ---------------------------------------------------------------------------

static EACH_TYPE_RESULT: OnceLock<CqlResult> = OnceLock::new();

fn each_type_result() -> &'static CqlResult {
    EACH_TYPE_RESULT.get_or_init(|| {
        let test_uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let columns = vec![
            CqlColumn { name: "ascii_col".to_string(),    type_name: "ascii".to_string() },
            CqlColumn { name: "boolean_col".to_string(),  type_name: "boolean".to_string() },
            CqlColumn { name: "bigint_col".to_string(),   type_name: "bigint".to_string() },
            CqlColumn { name: "blob_col".to_string(),     type_name: "blob".to_string() },
            CqlColumn { name: "counter_col".to_string(),  type_name: "counter".to_string() },
            CqlColumn { name: "double_col".to_string(),   type_name: "double".to_string() },
            CqlColumn { name: "duration_col".to_string(), type_name: "duration".to_string() },
            CqlColumn { name: "float_col".to_string(),    type_name: "float".to_string() },
            CqlColumn { name: "int_col".to_string(),      type_name: "int".to_string() },
            CqlColumn { name: "smallint_col".to_string(), type_name: "smallint".to_string() },
            CqlColumn { name: "tinyint_col".to_string(),  type_name: "tinyint".to_string() },
            CqlColumn { name: "timestamp_col".to_string(),type_name: "timestamp".to_string() },
            CqlColumn { name: "uuid_col".to_string(),     type_name: "uuid".to_string() },
            CqlColumn { name: "timeuuid_col".to_string(), type_name: "timeuuid".to_string() },
            CqlColumn { name: "inet_col".to_string(),     type_name: "inet".to_string() },
            CqlColumn { name: "date_col".to_string(),     type_name: "date".to_string() },
            CqlColumn { name: "time_col".to_string(),     type_name: "time".to_string() },
            CqlColumn { name: "text_col".to_string(),     type_name: "text".to_string() },
            CqlColumn { name: "varint_col".to_string(),   type_name: "varint".to_string() },
            CqlColumn { name: "list_col".to_string(),     type_name: "list<int>".to_string() },
            CqlColumn { name: "set_col".to_string(),      type_name: "set<text>".to_string() },
            CqlColumn { name: "map_col".to_string(),      type_name: "map<text,int>".to_string() },
            CqlColumn { name: "tuple_col".to_string(),    type_name: "tuple<int,text>".to_string() },
            CqlColumn { name: "null_col".to_string(),     type_name: "text".to_string() },
        ];

        let row = CqlRow {
            values: vec![
                CqlValue::Ascii("hello".to_string()),
                CqlValue::Boolean(true),
                CqlValue::BigInt(9_223_372_036_854_775_807),
                CqlValue::Blob(vec![0xde, 0xad, 0xbe, 0xef]),
                CqlValue::Counter(42),
                CqlValue::Double(std::f64::consts::PI),
                CqlValue::Duration { months: 1, days: 2, nanoseconds: 3_000_000_000 },
                CqlValue::Float(std::f32::consts::E),
                CqlValue::Int(2_147_483_647),
                CqlValue::SmallInt(32_767),
                CqlValue::TinyInt(127),
                CqlValue::Timestamp(1_700_000_000_000),
                CqlValue::Uuid(test_uuid),
                CqlValue::TimeUuid(test_uuid),
                CqlValue::Inet(IpAddr::from([127, 0, 0, 1])),
                CqlValue::Date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
                CqlValue::Time(NaiveTime::from_hms_opt(12, 30, 0).unwrap()),
                CqlValue::Text("world".to_string()),
                CqlValue::Varint(BigInt::from(123_456_789_012_345_678_i64)),
                CqlValue::List(vec![CqlValue::Int(1), CqlValue::Int(2), CqlValue::Int(3)]),
                CqlValue::Set(vec![
                    CqlValue::Text("a".to_string()),
                    CqlValue::Text("b".to_string()),
                ]),
                CqlValue::Map(vec![
                    (CqlValue::Text("k1".to_string()), CqlValue::Int(10)),
                    (CqlValue::Text("k2".to_string()), CqlValue::Int(20)),
                ]),
                CqlValue::Tuple(vec![
                    Some(CqlValue::Int(1)),
                    Some(CqlValue::Text("t".to_string())),
                ]),
                CqlValue::Null,
            ],
        };

        CqlResult {
            columns,
            rows: vec![row],
            has_rows: true,
            tracing_id: None,
            warnings: vec![],
        }
    })
}

fn bench_format_each_type(c: &mut Criterion) {
    let result = each_type_result();
    let color = no_color();

    c.bench_function("format_each_type", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(512);
            print_tabular(black_box(result), &color, &mut buf);
            buf
        })
    });
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_format_table,
    bench_format_json_100,
    bench_format_csv_100,
    bench_format_each_type,
);
criterion_main!(benches);

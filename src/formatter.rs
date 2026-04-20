//! Output formatting for CQL query results.
//!
//! Provides type-aware tabular formatting using comfy-table and expanded (vertical)
//! output mode. Paging is handled externally by the `minus` pager crate.
//! Mirrors the Python cqlsh output formatting behavior.

use std::io::Write;

use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};

use crate::colorizer::CqlColorizer;
use crate::driver::CqlResult;

/// Format and print query results in tabular format.
///
/// Uses comfy-table for proper column alignment. Columns render at natural width
/// (no terminal width constraint) — the pager handles horizontal scrolling.
/// When a colorizer is provided, values and headers are colored.
pub fn print_tabular(result: &CqlResult, colorizer: &CqlColorizer, writer: &mut dyn Write) {
    if result.columns.is_empty() {
        return;
    }

    if result.rows.is_empty() {
        writeln!(writer, "\n(0 rows)\n").ok();
        return;
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Disabled);
    table.load_preset(CQLSH_PRESET);

    // Add header row (magenta bold when colored, plain bold otherwise)
    let headers: Vec<Cell> = result
        .columns
        .iter()
        .map(|c| Cell::new(colorizer.colorize_header(&c.name)))
        .collect();
    table.set_header(headers);

    // Add data rows with type-aware alignment and coloring
    for row in &result.rows {
        let cells: Vec<Cell> = row
            .values
            .iter()
            .enumerate()
            .map(|(i, val)| {
                let display = colorizer.colorize_value(val);
                let mut cell = Cell::new(display);
                // Right-align numeric types to match Python cqlsh
                if is_numeric_type(&result.columns[i].type_name) {
                    cell = cell.set_alignment(CellAlignment::Right);
                }
                cell
            })
            .collect();
        table.add_row(cells);
    }

    writeln!(writer).ok();
    for line in format!("{table}").lines() {
        writeln!(writer, "{}", line.trim_end()).ok();
    }
    writeln!(writer).ok();
    let row_count = result.rows.len();
    writeln!(
        writer,
        "({} row{})",
        row_count,
        if row_count == 1 { "" } else { "s" }
    )
    .ok();
    writeln!(writer).ok();
}

/// Format and print query results in expanded (vertical) format.
///
/// Each row is printed as a block with `@ Row N` header, followed by
/// column_name | value pairs. Matches Python cqlsh `EXPAND ON` behavior.
pub fn print_expanded(result: &CqlResult, colorizer: &CqlColorizer, writer: &mut dyn Write) {
    if result.columns.is_empty() {
        return;
    }

    if result.rows.is_empty() {
        writeln!(writer, "\n(0 rows)\n").ok();
        return;
    }

    let max_col_width = result
        .columns
        .iter()
        .map(|c| c.name.len())
        .max()
        .unwrap_or(0);

    writeln!(writer).ok();

    for (row_idx, row) in result.rows.iter().enumerate() {
        writeln!(writer, "@ Row {}", row_idx + 1).ok();
        writeln!(writer, "{}", "-".repeat(max_col_width + 10)).ok();
        for (col_idx, col) in result.columns.iter().enumerate() {
            let value = row
                .get(col_idx)
                .map(|v| colorizer.colorize_value(v))
                .unwrap_or_else(|| colorizer.colorize_value(&crate::driver::types::CqlValue::Null));
            writeln!(
                writer,
                " {:>width$} | {}",
                colorizer.colorize_header(&col.name),
                value,
                width = max_col_width
            )
            .ok();
        }
        writeln!(writer).ok();
    }

    let row_count = result.rows.len();
    writeln!(
        writer,
        "({} row{})",
        row_count,
        if row_count == 1 { "" } else { "s" }
    )
    .ok();
    writeln!(writer).ok();
}

/// Format and print query results as a JSON array.
///
/// Each row becomes a JSON object mapping column names to values.
/// Matches the format produced by Python cqlsh `--json`.
/// NaN and Infinity float values are serialized as quoted strings since they
/// are not valid JSON numbers.
pub fn print_json(result: &CqlResult, writer: &mut dyn Write) {
    use crate::driver::types::CqlValue;

    if result.columns.is_empty() || result.rows.is_empty() {
        writeln!(writer, "[]").ok();
        return;
    }

    writeln!(writer, "[").ok();
    let last_row = result.rows.len() - 1;
    for (row_idx, row) in result.rows.iter().enumerate() {
        write!(writer, "  {{").ok();
        for (col_idx, col) in result.columns.iter().enumerate() {
            if col_idx > 0 {
                write!(writer, ", ").ok();
            }
            let val = row.get(col_idx).unwrap_or(&CqlValue::Null);
            write!(
                writer,
                "\"{}\": {}",
                json_escape_string(&col.name),
                cql_value_to_json(val)
            )
            .ok();
        }
        if row_idx < last_row {
            writeln!(writer, "}},").ok();
        } else {
            writeln!(writer, "}}").ok();
        }
    }
    writeln!(writer, "]").ok();
}

/// Escape a string for use as a JSON string value (without surrounding quotes).
fn json_escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Serialize a CqlValue to a JSON token.
fn cql_value_to_json(val: &crate::driver::types::CqlValue) -> String {
    use crate::driver::types::CqlValue;
    match val {
        CqlValue::Null | CqlValue::Unset => "null".to_string(),
        CqlValue::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
        CqlValue::Int(v) => v.to_string(),
        CqlValue::BigInt(v) | CqlValue::Counter(v) => v.to_string(),
        CqlValue::SmallInt(v) => v.to_string(),
        CqlValue::TinyInt(v) => v.to_string(),
        CqlValue::Float(v) => {
            if v.is_finite() {
                v.to_string()
            } else {
                format!("\"{}\"", val)
            }
        }
        CqlValue::Double(v) => {
            if v.is_finite() {
                v.to_string()
            } else {
                format!("\"{}\"", val)
            }
        }
        CqlValue::Decimal(v) => format!("\"{}\"", v),
        CqlValue::Varint(v) => format!("\"{}\"", v),
        CqlValue::Text(s) | CqlValue::Ascii(s) => {
            format!("\"{}\"", json_escape_string(s))
        }
        CqlValue::Uuid(u) | CqlValue::TimeUuid(u) => format!("\"{}\"", u),
        CqlValue::Inet(addr) => format!("\"{}\"", addr),
        CqlValue::Blob(bytes) => {
            let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            format!("\"0x{hex}\"")
        }
        CqlValue::Timestamp(_) | CqlValue::Date(_) | CqlValue::Time(_) => {
            format!("\"{}\"", val)
        }
        CqlValue::Duration {
            months,
            days,
            nanoseconds,
        } => format!("\"{months}mo{days}d{nanoseconds}ns\""),
        CqlValue::List(items) | CqlValue::Set(items) => {
            let elems: Vec<String> = items.iter().map(cql_value_to_json).collect();
            format!("[{}]", elems.join(", "))
        }
        CqlValue::Map(entries) => {
            let pairs: Vec<String> = entries
                .iter()
                .map(|(k, v)| {
                    let key = match k {
                        CqlValue::Text(s) | CqlValue::Ascii(s) => {
                            format!("\"{}\"", json_escape_string(s))
                        }
                        other => format!("\"{}\"", json_escape_string(&other.to_string())),
                    };
                    format!("{key}: {}", cql_value_to_json(v))
                })
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        CqlValue::Tuple(items) => {
            let elems: Vec<String> = items
                .iter()
                .map(|opt| opt.as_ref().map_or("null".to_string(), cql_value_to_json))
                .collect();
            format!("[{}]", elems.join(", "))
        }
        CqlValue::UserDefinedType { fields, .. } => {
            let pairs: Vec<String> = fields
                .iter()
                .map(|(name, v)| {
                    let json_val = v.as_ref().map_or("null".to_string(), cql_value_to_json);
                    format!("\"{}\": {json_val}", json_escape_string(name))
                })
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
    }
}

/// Format tracing session output matching Python cqlsh style.
///
/// Displays session metadata and a table of trace events sorted by elapsed time.
pub fn print_trace(
    trace: &crate::driver::TracingSession,
    colorizer: &CqlColorizer,
    writer: &mut dyn Write,
) {
    writeln!(writer).ok();
    writeln!(
        writer,
        "{} {}",
        colorizer.colorize_trace_label("Tracing session:"),
        trace.trace_id
    )
    .ok();
    writeln!(writer).ok();

    if let Some(ref request) = trace.request {
        writeln!(writer, " Request: {request}").ok();
    }
    if let Some(ref coordinator) = trace.coordinator {
        writeln!(writer, " Coordinator: {coordinator}").ok();
    }
    if let Some(duration) = trace.duration {
        writeln!(writer, " Duration: {} microseconds", duration).ok();
    }
    if let Some(ref started_at) = trace.started_at {
        writeln!(writer, " Started at: {started_at}").ok();
    }

    if !trace.events.is_empty() {
        writeln!(writer).ok();

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Disabled);
        table.load_preset(CQLSH_PRESET);
        table.set_header(vec![
            Cell::new(colorizer.colorize_header("activity")),
            Cell::new(colorizer.colorize_header("timestamp")),
            Cell::new(colorizer.colorize_header("source")),
            Cell::new(colorizer.colorize_header("source_elapsed")),
            Cell::new(colorizer.colorize_header("thread")),
        ]);

        for event in &trace.events {
            let elapsed_str = event
                .source_elapsed
                .map(|e| format!("{e}"))
                .unwrap_or_default();
            table.add_row(vec![
                Cell::new(event.activity.as_deref().unwrap_or("")),
                Cell::new(""),
                Cell::new(event.source.as_deref().unwrap_or("")),
                Cell::new(&elapsed_str).set_alignment(CellAlignment::Right),
                Cell::new(event.thread.as_deref().unwrap_or("")),
            ]);
        }

        writeln!(writer, "{table}").ok();
    }
    writeln!(writer).ok();
}

/// Check if a CQL type name represents a numeric type.
fn is_numeric_type(type_name: &str) -> bool {
    let lower = type_name.to_lowercase();
    matches!(
        lower.as_str(),
        "int"
            | "bigint"
            | "smallint"
            | "tinyint"
            | "float"
            | "double"
            | "decimal"
            | "varint"
            | "counter"
    ) || lower.contains("int")
        || lower.contains("float")
        || lower.contains("double")
        || lower.contains("decimal")
        || lower.contains("counter")
        || lower.contains("varint")
}

/// A comfy-table preset matching Python cqlsh's simple pipe-separated output.
///
/// Preset char positions (comfy-table v7):
///   0=LeftBorder, 1=RightBorder, 2=TopBorder, 3=BottomBorder,
///   4=LeftHeaderIntersection, 5=HeaderLines, 6=MiddleHeaderIntersections,
///   7=RightHeaderIntersection, 8=VerticalLines, 9=HorizontalLines,
///   10=MiddleIntersections, 11=LeftBorderIntersections,
///   12=RightBorderIntersections, 13=TopBorderIntersections,
///   14=BottomBorderIntersections, 15=TopLeftCorner, 16=TopRightCorner,
///   17=BottomLeftCorner, 18=BottomRightCorner
///
/// Example:
/// ```text
///  name | age | city
/// ------+-----+------
///  Alice | 30 | NYC
///  Bob   | 25 | LA
/// ```
//                    0123456789012345678
const CQLSH_PRESET: &str = "     -+ |          ";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::types::{CqlColumn, CqlResult, CqlRow, CqlValue};

    fn no_color() -> CqlColorizer {
        CqlColorizer::new(false)
    }

    fn sample_result() -> CqlResult {
        CqlResult {
            columns: vec![
                CqlColumn {
                    name: "name".to_string(),
                    type_name: "text".to_string(),
                },
                CqlColumn {
                    name: "age".to_string(),
                    type_name: "int".to_string(),
                },
            ],
            rows: vec![
                CqlRow {
                    values: vec![CqlValue::Text("Alice".to_string()), CqlValue::Int(30)],
                },
                CqlRow {
                    values: vec![CqlValue::Text("Bob".to_string()), CqlValue::Int(25)],
                },
            ],
            has_rows: true,
            tracing_id: None,
            warnings: vec![],
        }
    }

    #[test]
    fn tabular_output_contains_headers_and_rows() {
        let result = sample_result();
        let mut buf = Vec::new();
        print_tabular(&result, &no_color(), &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("name"));
        assert!(output.contains("age"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        assert!(output.contains("(2 rows)"));
    }

    #[test]
    fn expanded_output_shows_row_headers() {
        let result = sample_result();
        let mut buf = Vec::new();
        print_expanded(&result, &no_color(), &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("@ Row 1"));
        assert!(output.contains("@ Row 2"));
        assert!(output.contains("Alice"));
        assert!(output.contains("(2 rows)"));
    }

    #[test]
    fn tabular_empty_result_produces_no_output() {
        let result = CqlResult::empty();
        let mut buf = Vec::new();
        print_tabular(&result, &no_color(), &mut buf);
        assert!(buf.is_empty());
    }

    #[test]
    fn single_row_says_row_not_rows() {
        let result = CqlResult {
            columns: vec![CqlColumn {
                name: "id".to_string(),
                type_name: "int".to_string(),
            }],
            rows: vec![CqlRow {
                values: vec![CqlValue::Int(1)],
            }],
            has_rows: true,
            tracing_id: None,
            warnings: vec![],
        };
        let mut buf = Vec::new();
        print_tabular(&result, &no_color(), &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("(1 row)"));
        assert!(!output.contains("(1 rows)"));
    }

    #[test]
    fn numeric_type_detection() {
        assert!(is_numeric_type("int"));
        assert!(is_numeric_type("bigint"));
        assert!(is_numeric_type("float"));
        assert!(is_numeric_type("double"));
        assert!(is_numeric_type("decimal"));
        assert!(!is_numeric_type("text"));
        assert!(!is_numeric_type("uuid"));
        assert!(!is_numeric_type("boolean"));
    }

    #[test]
    fn tabular_row_separators_not_pipes() {
        let result = sample_result();
        let mut buf = Vec::new();
        print_tabular(&result, &no_color(), &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(
            !output.contains("||||"),
            "row separators should not contain pipe characters"
        );
        assert!(
            output.contains("-+-") || output.contains("---"),
            "header separator should use dashes"
        );
    }

    #[test]
    fn tabular_columns_separated_by_pipes() {
        let result = sample_result();
        let mut buf = Vec::new();
        print_tabular(&result, &no_color(), &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(
            output.contains("| "),
            "columns should be separated by pipes"
        );
    }

    #[test]
    fn trace_output_format() {
        use crate::driver::{TracingEvent, TracingSession};
        use std::collections::HashMap;

        let trace = TracingSession {
            trace_id: uuid::Uuid::nil(),
            client: Some("127.0.0.1".to_string()),
            command: Some("QUERY".to_string()),
            coordinator: Some("127.0.0.1".to_string()),
            duration: Some(1234),
            parameters: HashMap::new(),
            request: Some("SELECT * FROM test".to_string()),
            started_at: Some("2024-01-01 00:00:00".to_string()),
            events: vec![TracingEvent {
                activity: Some("Parsing request".to_string()),
                source: Some("127.0.0.1".to_string()),
                source_elapsed: Some(100),
                thread: Some("Native-Transport-1".to_string()),
            }],
        };

        let mut buf = Vec::new();
        print_trace(&trace, &no_color(), &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Tracing session:"));
        assert!(output.contains("SELECT * FROM test"));
        assert!(output.contains("1234 microseconds"));
        assert!(output.contains("Parsing request"));
    }

    #[test]
    fn json_escape_special_chars() {
        assert_eq!(json_escape_string("hello"), "hello");
        assert_eq!(json_escape_string("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(json_escape_string("back\\slash"), "back\\\\slash");
        assert_eq!(json_escape_string("new\nline"), "new\\nline");
        assert_eq!(json_escape_string("tab\there"), "tab\\there");
        assert_eq!(json_escape_string("cr\rhere"), "cr\\rhere");
        assert_eq!(json_escape_string("\x01"), "\\u0001");
    }

    #[test]
    fn cql_value_to_json_scalars() {
        use crate::driver::types::CqlValue;
        assert_eq!(cql_value_to_json(&CqlValue::Null), "null");
        assert_eq!(cql_value_to_json(&CqlValue::Unset), "null");
        assert_eq!(cql_value_to_json(&CqlValue::Boolean(true)), "true");
        assert_eq!(cql_value_to_json(&CqlValue::Boolean(false)), "false");
        assert_eq!(cql_value_to_json(&CqlValue::Int(42)), "42");
        assert_eq!(cql_value_to_json(&CqlValue::BigInt(-100)), "-100");
        assert_eq!(cql_value_to_json(&CqlValue::SmallInt(7)), "7");
        assert_eq!(cql_value_to_json(&CqlValue::TinyInt(-1)), "-1");
        assert_eq!(cql_value_to_json(&CqlValue::Counter(99)), "99");
    }

    #[test]
    fn cql_value_to_json_strings() {
        use crate::driver::types::CqlValue;
        assert_eq!(
            cql_value_to_json(&CqlValue::Text("hello".to_string())),
            "\"hello\""
        );
        assert_eq!(
            cql_value_to_json(&CqlValue::Ascii("world".to_string())),
            "\"world\""
        );
        assert_eq!(
            cql_value_to_json(&CqlValue::Text("say \"hi\"".to_string())),
            "\"say \\\"hi\\\"\""
        );
    }

    #[test]
    fn cql_value_to_json_float_special() {
        use crate::driver::types::CqlValue;
        assert_eq!(cql_value_to_json(&CqlValue::Float(1.5)), "1.5");
        assert_eq!(cql_value_to_json(&CqlValue::Double(2.5)), "2.5");

        let nan_json = cql_value_to_json(&CqlValue::Float(f32::NAN));
        assert!(nan_json.starts_with('"') && nan_json.ends_with('"'));
        let inf_json = cql_value_to_json(&CqlValue::Double(f64::INFINITY));
        assert!(inf_json.starts_with('"') && inf_json.ends_with('"'));
    }

    #[test]
    fn cql_value_to_json_uuid_inet_blob() {
        use crate::driver::types::CqlValue;
        use std::net::IpAddr;
        assert_eq!(
            cql_value_to_json(&CqlValue::Uuid(uuid::Uuid::nil())),
            "\"00000000-0000-0000-0000-000000000000\""
        );
        assert_eq!(
            cql_value_to_json(&CqlValue::Inet("127.0.0.1".parse::<IpAddr>().unwrap())),
            "\"127.0.0.1\""
        );
        assert_eq!(
            cql_value_to_json(&CqlValue::Blob(vec![0xca, 0xfe])),
            "\"0xcafe\""
        );
    }

    #[test]
    fn cql_value_to_json_collections() {
        use crate::driver::types::CqlValue;
        let list = CqlValue::List(vec![CqlValue::Int(1), CqlValue::Int(2)]);
        assert_eq!(cql_value_to_json(&list), "[1, 2]");

        let set = CqlValue::Set(vec![CqlValue::Text("a".to_string())]);
        assert_eq!(cql_value_to_json(&set), "[\"a\"]");

        let map = CqlValue::Map(vec![(CqlValue::Text("key".to_string()), CqlValue::Int(42))]);
        assert_eq!(cql_value_to_json(&map), "{\"key\": 42}");

        let map2 = CqlValue::Map(vec![(CqlValue::Int(1), CqlValue::Boolean(true))]);
        assert_eq!(cql_value_to_json(&map2), "{\"1\": true}");
    }

    #[test]
    fn cql_value_to_json_tuple_and_udt() {
        use crate::driver::types::CqlValue;
        let tuple = CqlValue::Tuple(vec![Some(CqlValue::Int(1)), None]);
        assert_eq!(cql_value_to_json(&tuple), "[1, null]");

        let udt = CqlValue::UserDefinedType {
            keyspace: "ks".to_string(),
            type_name: "t".to_string(),
            fields: vec![
                (
                    "name".to_string(),
                    Some(CqlValue::Text("Alice".to_string())),
                ),
                ("age".to_string(), None),
            ],
        };
        assert_eq!(
            cql_value_to_json(&udt),
            "{\"name\": \"Alice\", \"age\": null}"
        );
    }

    #[test]
    fn cql_value_to_json_duration_decimal_varint() {
        use crate::driver::types::CqlValue;
        use bigdecimal::BigDecimal;
        use num_bigint::BigInt;
        use std::str::FromStr;

        let dur = CqlValue::Duration {
            months: 1,
            days: 2,
            nanoseconds: 3,
        };
        assert_eq!(cql_value_to_json(&dur), "\"1mo2d3ns\"");

        let dec = CqlValue::Decimal(BigDecimal::from_str("3.14").unwrap());
        assert_eq!(cql_value_to_json(&dec), "\"3.14\"");

        let varint = CqlValue::Varint(BigInt::from(12345));
        assert_eq!(cql_value_to_json(&varint), "\"12345\"");
    }

    #[test]
    fn print_json_empty_result() {
        let result = CqlResult::empty();
        let mut buf = Vec::new();
        print_json(&result, &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output.trim(), "[]");
    }

    #[test]
    fn print_json_single_row() {
        let result = CqlResult {
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
            rows: vec![CqlRow {
                values: vec![CqlValue::Int(1), CqlValue::Text("Alice".to_string())],
            }],
            has_rows: true,
            tracing_id: None,
            warnings: vec![],
        };
        let mut buf = Vec::new();
        print_json(&result, &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("\"id\": 1"));
        assert!(output.contains("\"name\": \"Alice\""));
        assert!(output.starts_with("[\n"));
        assert!(output.trim().ends_with("]"));
    }

    #[test]
    fn print_json_multiple_rows_has_commas() {
        let result = CqlResult {
            columns: vec![CqlColumn {
                name: "v".to_string(),
                type_name: "int".to_string(),
            }],
            rows: vec![
                CqlRow {
                    values: vec![CqlValue::Int(1)],
                },
                CqlRow {
                    values: vec![CqlValue::Int(2)],
                },
            ],
            has_rows: true,
            tracing_id: None,
            warnings: vec![],
        };
        let mut buf = Vec::new();
        print_json(&result, &mut buf);
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines[1].ends_with("},"));
        assert!(lines[2].ends_with("}"));
    }

    #[test]
    fn wide_table_not_truncated() {
        let columns: Vec<CqlColumn> = (0..20)
            .map(|i| CqlColumn {
                name: format!("column_{i}"),
                type_name: "text".to_string(),
            })
            .collect();
        let rows = vec![CqlRow {
            values: (0..20)
                .map(|i| CqlValue::Text(format!("value_{i}_with_long_content")))
                .collect(),
        }];
        let result = CqlResult {
            columns,
            rows,
            has_rows: true,
            tracing_id: None,
            warnings: vec![],
        };
        let mut buf = Vec::new();
        print_tabular(&result, &no_color(), &mut buf);
        let output = String::from_utf8(buf).unwrap();
        // All 20 columns should appear on the header line
        assert!(output.contains("column_0"));
        assert!(output.contains("column_19"));
        // Values should not be truncated
        assert!(output.contains("value_19_with_long_content"));
    }
}

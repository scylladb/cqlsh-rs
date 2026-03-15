//! Output formatting for CQL query results.
//!
//! Provides type-aware tabular formatting using comfy-table, expanded (vertical)
//! output mode, and pagination with a `--More--` prompt. Mirrors the Python cqlsh
//! output formatting behavior.

use std::io::{self, Read, Write};

use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
use terminal_size::{terminal_size, Width};

use crate::driver::CqlResult;

fn terminal_width() -> Option<u16> {
    terminal_size().map(|(Width(w), _)| w)
}

/// Format and print query results in tabular format.
///
/// Uses comfy-table for proper column alignment and Unicode box-drawing.
/// Matches Python cqlsh output style with column headers and row counts.
pub fn print_tabular(result: &CqlResult, writer: &mut dyn Write) {
    if result.columns.is_empty() {
        return;
    }

    if result.rows.is_empty() {
        writeln!(writer, "\n(0 rows)\n").ok();
        return;
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    if let Some(w) = terminal_width() {
        table.set_width(w);
    }

    // Disable borders to match Python cqlsh's simple pipe-separated output
    table.load_preset(CQLSH_PRESET);

    // Add header row
    let headers: Vec<Cell> = result
        .columns
        .iter()
        .map(|c| Cell::new(&c.name).add_attribute(Attribute::Bold))
        .collect();
    table.set_header(headers);

    // Add data rows with type-aware alignment
    for row in &result.rows {
        let cells: Vec<Cell> = row
            .values
            .iter()
            .enumerate()
            .map(|(i, val)| {
                let display = val.to_string();
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
    writeln!(writer, "{table}").ok();
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
pub fn print_expanded(result: &CqlResult, writer: &mut dyn Write) {
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
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string());
            writeln!(
                writer,
                " {:>width$} | {}",
                col.name,
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

/// Display results with pagination, pausing every `page_size` rows.
///
/// Shows a `---MORE---` prompt between pages. The user can press any key
/// to continue, or 'q' to stop.
pub fn print_paged(result: &CqlResult, page_size: usize, expand: bool, writer: &mut dyn Write) {
    if result.columns.is_empty() {
        return;
    }

    if expand {
        // For expanded output, page by row count
        print_expanded_paged(result, page_size, writer);
    } else {
        print_tabular_paged(result, page_size, writer);
    }
}

/// Print tabular output with pagination.
fn print_tabular_paged(result: &CqlResult, page_size: usize, writer: &mut dyn Write) {
    let total_rows = result.rows.len();
    if total_rows <= page_size {
        print_tabular(result, writer);
        return;
    }

    // Print header once
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    if let Some(w) = terminal_width() {
        table.set_width(w);
    }
    table.load_preset(CQLSH_PRESET);

    let headers: Vec<Cell> = result
        .columns
        .iter()
        .map(|c| Cell::new(&c.name).add_attribute(Attribute::Bold))
        .collect();
    table.set_header(headers);

    // Build the full table to get the header, then print rows page by page
    for (chunk_idx, chunk) in result.rows.chunks(page_size).enumerate() {
        if chunk_idx > 0 && !show_more_prompt() {
            break;
        }

        let mut page_table = Table::new();
        page_table.set_content_arrangement(ContentArrangement::Dynamic);
        if let Some(w) = terminal_width() {
            page_table.set_width(w);
        }
        page_table.load_preset(CQLSH_PRESET);

        if chunk_idx == 0 {
            let headers: Vec<Cell> = result
                .columns
                .iter()
                .map(|c| Cell::new(&c.name).add_attribute(Attribute::Bold))
                .collect();
            page_table.set_header(headers);
            writeln!(writer).ok();
        }

        for row in chunk {
            let cells: Vec<Cell> = row
                .values
                .iter()
                .enumerate()
                .map(|(i, val)| {
                    let display = val.to_string();
                    let mut cell = Cell::new(display);
                    if is_numeric_type(&result.columns[i].type_name) {
                        cell = cell.set_alignment(CellAlignment::Right);
                    }
                    cell
                })
                .collect();
            page_table.add_row(cells);
        }

        writeln!(writer, "{page_table}").ok();
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

/// Print expanded output with pagination.
fn print_expanded_paged(result: &CqlResult, page_size: usize, writer: &mut dyn Write) {
    let total_rows = result.rows.len();
    if total_rows <= page_size {
        print_expanded(result, writer);
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
        if row_idx > 0 && row_idx % page_size == 0 && !show_more_prompt() {
            break;
        }

        writeln!(writer, "@ Row {}", row_idx + 1).ok();
        writeln!(writer, "{}", "-".repeat(max_col_width + 10)).ok();
        for (col_idx, col) in result.columns.iter().enumerate() {
            let value = row
                .get(col_idx)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string());
            writeln!(
                writer,
                " {:>width$} | {}",
                col.name,
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

/// Show `---MORE---` prompt and wait for user input.
/// Returns true to continue, false to stop.
fn show_more_prompt() -> bool {
    eprint!("---MORE---");
    let _ = io::stderr().flush();

    // Read a single byte from stdin in raw mode
    if crossterm::terminal::enable_raw_mode().is_ok() {
        let mut buf = [0u8; 1];
        let result = io::stdin().read_exact(&mut buf);
        let _ = crossterm::terminal::disable_raw_mode();
        // Clear the ---MORE--- line
        eprint!("\r          \r");
        let _ = io::stderr().flush();

        if result.is_ok() {
            // 'q' or 'Q' to quit, Ctrl-C to quit
            return buf[0] != b'q' && buf[0] != b'Q' && buf[0] != 3;
        }
    }
    true
}

/// Format tracing session output matching Python cqlsh style.
///
/// Displays session metadata and a table of trace events sorted by elapsed time.
pub fn print_trace(trace: &crate::driver::TracingSession, writer: &mut dyn Write) {
    writeln!(writer).ok();
    writeln!(writer, "Tracing session: {}", trace.trace_id).ok();
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
        table.set_content_arrangement(ContentArrangement::Dynamic);
        if let Some(w) = terminal_width() {
            table.set_width(w);
        }
        table.load_preset(CQLSH_PRESET);
        table.set_header(vec![
            Cell::new("activity").add_attribute(Attribute::Bold),
            Cell::new("timestamp").add_attribute(Attribute::Bold),
            Cell::new("source").add_attribute(Attribute::Bold),
            Cell::new("source_elapsed").add_attribute(Attribute::Bold),
            Cell::new("thread").add_attribute(Attribute::Bold),
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
        print_tabular(&result, &mut buf);
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
        print_expanded(&result, &mut buf);
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
        print_tabular(&result, &mut buf);
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
        print_tabular(&result, &mut buf);
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
        print_tabular(&result, &mut buf);
        let output = String::from_utf8(buf).unwrap();
        // Bug: row separators were `||||` instead of proper formatting.
        // The header separator should use `-` and `+`, not `|`.
        assert!(!output.contains("||||"), "row separators should not contain pipe characters");
        // Header separator should exist with dashes and plus signs
        assert!(output.contains("-+-") || output.contains("---"), "header separator should use dashes");
    }

    #[test]
    fn tabular_columns_separated_by_pipes() {
        let result = sample_result();
        let mut buf = Vec::new();
        print_tabular(&result, &mut buf);
        let output = String::from_utf8(buf).unwrap();
        // Data rows should have `|` as column separator
        assert!(output.contains("| "), "columns should be separated by pipes");
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
        print_trace(&trace, &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Tracing session:"));
        assert!(output.contains("SELECT * FROM test"));
        assert!(output.contains("1234 microseconds"));
        assert!(output.contains("Parsing request"));
    }
}

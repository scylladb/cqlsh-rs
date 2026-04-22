//! Output formatting for CQL query results.
//!
//! Provides type-aware tabular formatting using comfy-table and expanded (vertical)
//! output mode. Paging is handled externally by the `minus` pager crate.
//! Mirrors the Python cqlsh output formatting behavior.

use std::io::Write;

use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};

use crate::colorizer::CqlColorizer;
use crate::driver::{CqlColumn, CqlResult, CqlRow};

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

/// A streaming table formatter that computes column widths from the first page
/// of results and then formats subsequent rows incrementally.
pub struct StreamingTableFormatter<'w> {
    columns: Vec<CqlColumn>,
    colorizer: &'w CqlColorizer,
    writer: &'w mut dyn Write,
    /// Buffer for first-page rows used to compute column widths
    first_page_buffer: Vec<CqlRow>,
    /// Maximum number of rows to buffer for width computation
    page_size: usize,
    /// Whether the header has been written (first page flushed)
    header_written: bool,
    /// Computed column widths (set after first page flush)
    col_widths: Vec<usize>,
    /// Total rows written
    row_count: usize,
    /// Whether we're in expanded mode
    expanded: bool,
}

impl<'w> StreamingTableFormatter<'w> {
    /// Create a new streaming formatter in tabular mode.
    pub fn new(
        columns: Vec<CqlColumn>,
        colorizer: &'w CqlColorizer,
        writer: &'w mut dyn Write,
        page_size: usize,
    ) -> Self {
        Self {
            columns,
            colorizer,
            writer,
            first_page_buffer: Vec::with_capacity(page_size),
            page_size,
            header_written: false,
            col_widths: Vec::new(),
            row_count: 0,
            expanded: false,
        }
    }

    /// Create a new streaming formatter in expanded mode.
    pub fn new_expanded(
        columns: Vec<CqlColumn>,
        colorizer: &'w CqlColorizer,
        writer: &'w mut dyn Write,
    ) -> Self {
        Self {
            columns,
            colorizer,
            writer,
            first_page_buffer: Vec::new(),
            page_size: 0,
            header_written: true, // no header needed for expanded
            col_widths: Vec::new(),
            row_count: 0,
            expanded: true,
        }
    }

    /// Add a row to the formatter. Returns Err if writing fails (e.g. broken pipe).
    pub fn add_row(&mut self, row: CqlRow) -> std::io::Result<()> {
        self.row_count += 1;

        if self.expanded {
            return self.write_expanded_row(&row);
        }

        if !self.header_written {
            self.first_page_buffer.push(row);
            if self.first_page_buffer.len() >= self.page_size {
                self.flush_first_page()?;
            }
            return Ok(());
        }

        self.write_row(&row)
    }

    pub fn flush_writer(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

    /// Finish the stream: flush any buffered rows and write the footer.
    /// Returns the total row count.
    pub fn finish(mut self) -> std::io::Result<usize> {
        if !self.header_written && !self.first_page_buffer.is_empty() {
            self.flush_first_page()?;
        }

        if self.row_count == 0 {
            writeln!(self.writer, "\n(0 rows)\n")?;
        } else {
            writeln!(self.writer)?;
            writeln!(
                self.writer,
                "({} row{})",
                self.row_count,
                if self.row_count == 1 { "" } else { "s" }
            )?;
            writeln!(self.writer)?;
        }

        Ok(self.row_count)
    }

    /// Compute column widths from the first page buffer and flush everything.
    fn flush_first_page(&mut self) -> std::io::Result<()> {
        // Compute widths: max of header name and all values in first page
        self.col_widths = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let header_width = col.name.len();
                let max_val_width = self
                    .first_page_buffer
                    .iter()
                    .map(|row| {
                        row.values
                            .get(i)
                            .map(|v| format_value_plain(v).len())
                            .unwrap_or(0)
                    })
                    .max()
                    .unwrap_or(0);
                header_width.max(max_val_width)
            })
            .collect();

        self.header_written = true;

        // Write header
        writeln!(self.writer)?;
        self.write_separator()?;
        self.write_header_row()?;
        self.write_separator()?;

        // Write buffered rows
        let buffered: Vec<CqlRow> = std::mem::take(&mut self.first_page_buffer);
        for row in &buffered {
            self.write_row(row)?;
        }

        Ok(())
    }

    fn write_separator(&mut self) -> std::io::Result<()> {
        let parts: Vec<String> = self.col_widths.iter().map(|w| "-".repeat(*w + 2)).collect();
        writeln!(self.writer, "+{}+", parts.join("+"))
    }

    fn write_header_row(&mut self) -> std::io::Result<()> {
        let cells: Vec<String> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let width = self.col_widths[i];
                let name = self.colorizer.colorize_header(&col.name);
                // Pad based on raw name length (not colored length)
                let padding = width.saturating_sub(col.name.len());
                format!(" {}{} ", name, " ".repeat(padding))
            })
            .collect();
        writeln!(self.writer, "|{}|", cells.join("|"))
    }

    fn write_row(&mut self, row: &CqlRow) -> std::io::Result<()> {
        let cells: Vec<String> = row
            .values
            .iter()
            .enumerate()
            .map(|(i, val)| {
                let width = self.col_widths.get(i).copied().unwrap_or(10);
                let display = self.colorizer.colorize_value(val);
                let plain_len = format_value_plain(val).len();
                let padding = width.saturating_sub(plain_len);
                if is_numeric_type(
                    self.columns
                        .get(i)
                        .map(|c| c.type_name.as_str())
                        .unwrap_or(""),
                ) {
                    // Right-align numeric
                    format!(" {}{} ", " ".repeat(padding), display)
                } else {
                    format!(" {}{} ", display, " ".repeat(padding))
                }
            })
            .collect();
        writeln!(self.writer, "|{}|", cells.join("|"))
    }

    fn write_expanded_row(&mut self, row: &CqlRow) -> std::io::Result<()> {
        let max_col_width = self.columns.iter().map(|c| c.name.len()).max().unwrap_or(0);

        writeln!(self.writer, "@ Row {}", self.row_count)?;
        writeln!(self.writer, "{}", "-".repeat(max_col_width + 10))?;
        for (col_idx, col) in self.columns.iter().enumerate() {
            let value = row
                .values
                .get(col_idx)
                .map(|v| self.colorizer.colorize_value(v))
                .unwrap_or_else(|| {
                    self.colorizer
                        .colorize_value(&crate::driver::types::CqlValue::Null)
                });
            writeln!(
                self.writer,
                " {:>width$} | {}",
                self.colorizer.colorize_header(&col.name),
                value,
                width = max_col_width
            )?;
        }
        writeln!(self.writer)?;
        Ok(())
    }
}

/// Format a CqlValue to plain text (no ANSI colors) for width calculation.
fn format_value_plain(val: &crate::driver::types::CqlValue) -> String {
    use crate::driver::types::CqlValue;
    match val {
        CqlValue::Null => "null".to_string(),
        CqlValue::Text(s) | CqlValue::Ascii(s) => s.clone(),
        CqlValue::Boolean(b) => if *b { "True" } else { "False" }.to_string(),
        CqlValue::Int(n) => n.to_string(),
        CqlValue::BigInt(n) => n.to_string(),
        CqlValue::SmallInt(n) => n.to_string(),
        CqlValue::TinyInt(n) => n.to_string(),
        CqlValue::Float(n) => format!("{}", n),
        CqlValue::Double(n) => format!("{}", n),
        CqlValue::Uuid(u) | CqlValue::TimeUuid(u) => u.to_string(),
        _ => format!("{}", val),
    }
}

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

    fn make_col(name: &str, type_name: &str) -> CqlColumn {
        CqlColumn {
            name: name.to_string(),
            type_name: type_name.to_string(),
        }
    }

    fn make_row(values: Vec<CqlValue>) -> CqlRow {
        CqlRow { values }
    }

    #[test]
    fn streaming_finish_zero_rows() {
        let cols = vec![make_col("id", "int")];
        let color = no_color();
        let mut buf: Vec<u8> = Vec::new();
        let fmt = StreamingTableFormatter::new(cols, &color, &mut buf, 100);
        fmt.finish().unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("(0 rows)"));
    }

    #[test]
    fn streaming_finish_one_row_singular() {
        let cols = vec![make_col("id", "int")];
        let color = no_color();
        let mut buf: Vec<u8> = Vec::new();
        let mut fmt = StreamingTableFormatter::new(cols, &color, &mut buf, 100);
        fmt.add_row(make_row(vec![CqlValue::Int(1)])).unwrap();
        fmt.finish().unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("(1 row)"));
        assert!(!output.contains("(1 rows)"));
    }

    #[test]
    fn streaming_finish_multiple_rows_plural() {
        let cols = vec![make_col("id", "int")];
        let color = no_color();
        let mut buf: Vec<u8> = Vec::new();
        let mut fmt = StreamingTableFormatter::new(cols, &color, &mut buf, 100);
        fmt.add_row(make_row(vec![CqlValue::Int(1)])).unwrap();
        fmt.add_row(make_row(vec![CqlValue::Int(2)])).unwrap();
        fmt.add_row(make_row(vec![CqlValue::Int(3)])).unwrap();
        fmt.finish().unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("(3 rows)"));
    }

    #[test]
    fn streaming_tabular_buffered_rows_flushed_on_finish() {
        let cols = vec![make_col("name", "text"), make_col("age", "int")];
        let color = no_color();
        let mut buf: Vec<u8> = Vec::new();
        let mut fmt = StreamingTableFormatter::new(cols, &color, &mut buf, 100);
        fmt.add_row(make_row(vec![
            CqlValue::Text("Alice".to_string()),
            CqlValue::Int(30),
        ]))
        .unwrap();
        fmt.add_row(make_row(vec![
            CqlValue::Text("Bob".to_string()),
            CqlValue::Int(25),
        ]))
        .unwrap();
        fmt.finish().unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("name"));
        assert!(output.contains("age"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        assert!(output.contains("(2 rows)"));
    }

    #[test]
    fn streaming_flushes_first_page_at_page_size() {
        let cols = vec![make_col("id", "int")];
        let color = no_color();
        let mut buf: Vec<u8> = Vec::new();
        let page_size = 3;
        let mut fmt = StreamingTableFormatter::new(cols, &color, &mut buf, page_size);
        for i in 0..page_size {
            fmt.add_row(make_row(vec![CqlValue::Int(i as i32)]))
                .unwrap();
        }
        fmt.finish().unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("id"));
        assert!(output.contains("(3 rows)"));
    }

    #[test]
    fn streaming_post_flush_rows_written_directly() {
        let cols = vec![make_col("id", "int")];
        let color = no_color();
        let mut buf: Vec<u8> = Vec::new();
        let page_size = 2;
        let mut fmt = StreamingTableFormatter::new(cols, &color, &mut buf, page_size);
        fmt.add_row(make_row(vec![CqlValue::Int(1)])).unwrap();
        fmt.add_row(make_row(vec![CqlValue::Int(2)])).unwrap();
        fmt.add_row(make_row(vec![CqlValue::Int(3)])).unwrap();
        fmt.finish().unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("(3 rows)"));
    }

    #[test]
    fn streaming_flush_writer_succeeds() {
        let cols = vec![make_col("id", "int")];
        let color = no_color();
        let mut buf: Vec<u8> = Vec::new();
        let mut fmt = StreamingTableFormatter::new(cols, &color, &mut buf, 100);
        assert!(fmt.flush_writer().is_ok());
    }

    #[test]
    fn streaming_expanded_mode_writes_row_headers() {
        let cols = vec![make_col("name", "text"), make_col("age", "int")];
        let color = no_color();
        let mut buf: Vec<u8> = Vec::new();
        let mut fmt = StreamingTableFormatter::new_expanded(cols, &color, &mut buf);
        fmt.add_row(make_row(vec![
            CqlValue::Text("Alice".to_string()),
            CqlValue::Int(30),
        ]))
        .unwrap();
        fmt.add_row(make_row(vec![
            CqlValue::Text("Bob".to_string()),
            CqlValue::Int(25),
        ]))
        .unwrap();
        fmt.finish().unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("@ Row 1"));
        assert!(output.contains("@ Row 2"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        assert!(output.contains("(2 rows)"));
    }

    #[test]
    fn streaming_expanded_zero_rows_footer() {
        let cols = vec![make_col("id", "int")];
        let color = no_color();
        let mut buf: Vec<u8> = Vec::new();
        let fmt = StreamingTableFormatter::new_expanded(cols, &color, &mut buf);
        fmt.finish().unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("(0 rows)"));
    }

    #[test]
    fn streaming_numeric_right_aligned() {
        let cols = vec![make_col("name", "text"), make_col("score", "int")];
        let color = no_color();
        let mut buf: Vec<u8> = Vec::new();
        let page_size = 1;
        let mut fmt = StreamingTableFormatter::new(cols, &color, &mut buf, page_size);
        fmt.add_row(make_row(vec![
            CqlValue::Text("Alice".to_string()),
            CqlValue::Int(42),
        ]))
        .unwrap();
        fmt.finish().unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("42"));
        assert!(output.contains("Alice"));
    }

    #[test]
    fn streaming_write_error_propagates() {
        struct FailWriter;
        impl Write for FailWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "broken",
                ))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "broken",
                ))
            }
        }

        let cols = vec![make_col("id", "int")];
        let color = no_color();
        let mut writer = FailWriter;
        let mut fmt = StreamingTableFormatter::new(cols, &color, &mut writer, 1);
        let result = fmt.add_row(make_row(vec![CqlValue::Int(1)]));
        assert!(result.is_err());
    }
}

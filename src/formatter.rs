//! Result formatting for CQL query output.
//!
//! Provides tabular display matching Python cqlsh's default output format,
//! with column-aligned text and row separators.

use std::io::{self, Write};

use crate::driver::types::{CqlResult, CqlValue};

/// Format and print a CQL result set to the given writer.
///
/// Output format matches Python cqlsh:
/// ```text
///  col1 | col2 | col3
/// ------+------+------
///  val  | val  | val
///  val  | val  | val
///
/// (2 rows)
/// ```
pub fn print_result<W: Write>(writer: &mut W, result: &CqlResult) -> io::Result<()> {
    if !result.has_rows {
        return Ok(());
    }

    if result.columns.is_empty() {
        return Ok(());
    }

    // Calculate column widths
    let num_cols = result.columns.len();
    let mut widths: Vec<usize> = result.columns.iter().map(|c| c.name.len()).collect();

    let formatted_rows: Vec<Vec<String>> = result
        .rows
        .iter()
        .map(|row| {
            (0..num_cols)
                .map(|i| {
                    row.get(i)
                        .map(format_value)
                        .unwrap_or_else(|| "null".to_string())
                })
                .collect()
        })
        .collect();

    for row in &formatted_rows {
        for (i, val) in row.iter().enumerate() {
            if val.len() > widths[i] {
                widths[i] = val.len();
            }
        }
    }

    // Print header
    let header: Vec<String> = result
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| format!(" {:width$}", c.name, width = widths[i]))
        .collect();
    writeln!(writer, "{}", header.join(" |"))?;

    // Print separator
    let sep: Vec<String> = widths.iter().map(|w| "-".repeat(w + 2)).collect();
    writeln!(writer, "{}", sep.join("+"))?;

    // Print rows
    for row in &formatted_rows {
        let cells: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, val)| format!(" {:width$}", val, width = widths[i]))
            .collect();
        writeln!(writer, "{}", cells.join(" |"))?;
    }

    // Print row count
    let count = result.rows.len();
    writeln!(
        writer,
        "\n({} {})",
        count,
        if count == 1 { "row" } else { "rows" }
    )?;

    Ok(())
}

/// Print warnings from a query result.
pub fn print_warnings<W: Write>(writer: &mut W, result: &CqlResult) -> io::Result<()> {
    for warning in &result.warnings {
        writeln!(writer, "Warnings :")?;
        writeln!(writer, "{warning}")?;
    }
    Ok(())
}

/// Format a single CQL value for display.
fn format_value(value: &CqlValue) -> String {
    match value {
        CqlValue::Null => "null".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::types::{CqlColumn, CqlResult, CqlRow, CqlValue};

    fn make_result(col_names: &[&str], rows: Vec<Vec<CqlValue>>) -> CqlResult {
        let columns = col_names
            .iter()
            .map(|name| CqlColumn {
                name: name.to_string(),
                type_name: "text".to_string(),
            })
            .collect();
        let cql_rows = rows.into_iter().map(|values| CqlRow { values }).collect();
        CqlResult {
            columns,
            rows: cql_rows,
            has_rows: true,
            tracing_id: None,
            warnings: Vec::new(),
        }
    }

    #[test]
    fn format_single_row() {
        let result = make_result(
            &["id", "name"],
            vec![vec![CqlValue::Int(1), CqlValue::Text("Alice".to_string())]],
        );
        let mut buf = Vec::new();
        print_result(&mut buf, &result).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains(" id"));
        assert!(output.contains("name"));
        assert!(output.contains("1"));
        assert!(output.contains("Alice"));
        assert!(output.contains("(1 row)"));
    }

    #[test]
    fn format_multiple_rows() {
        let result = make_result(
            &["x"],
            vec![
                vec![CqlValue::Int(1)],
                vec![CqlValue::Int(2)],
                vec![CqlValue::Int(3)],
            ],
        );
        let mut buf = Vec::new();
        print_result(&mut buf, &result).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("(3 rows)"));
    }

    #[test]
    fn format_empty_result() {
        let result = CqlResult::empty();
        let mut buf = Vec::new();
        print_result(&mut buf, &result).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn format_null_values() {
        let result = make_result(&["col"], vec![vec![CqlValue::Null]]);
        let mut buf = Vec::new();
        print_result(&mut buf, &result).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("null"));
    }

    #[test]
    fn format_wide_columns() {
        let result = make_result(
            &["a", "very_long_column_name"],
            vec![vec![
                CqlValue::Text("short".to_string()),
                CqlValue::Text("x".to_string()),
            ]],
        );
        let mut buf = Vec::new();
        print_result(&mut buf, &result).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // The separator should be wide enough for the long column name
        assert!(output.contains("very_long_column_name"));
        // Values should be padded
        let lines: Vec<&str> = output.lines().collect();
        // All data lines should have the same visual width
        assert_eq!(lines[0].len(), lines[2].len());
    }

    #[test]
    fn format_separator_uses_plus() {
        let result = make_result(&["a", "b"], vec![vec![CqlValue::Int(1), CqlValue::Int(2)]]);
        let mut buf = Vec::new();
        print_result(&mut buf, &result).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let sep_line = output.lines().nth(1).unwrap();
        assert!(sep_line.contains('+'));
        assert!(sep_line.contains('-'));
    }

    #[test]
    fn format_warnings() {
        let mut result = CqlResult::empty();
        result.warnings = vec!["Something is deprecated".to_string()];
        let mut buf = Vec::new();
        print_warnings(&mut buf, &result).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Warnings"));
        assert!(output.contains("Something is deprecated"));
    }

    #[test]
    fn format_various_types() {
        let result = make_result(
            &["bool", "blob", "uuid_col"],
            vec![vec![
                CqlValue::Boolean(true),
                CqlValue::Blob(vec![0xca, 0xfe]),
                CqlValue::Uuid(uuid::Uuid::nil()),
            ]],
        );
        let mut buf = Vec::new();
        print_result(&mut buf, &result).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("True"));
        assert!(output.contains("0xcafe"));
        assert!(output.contains("00000000-0000-0000-0000-000000000000"));
    }
}

//! Non-interactive execution modes for cqlsh-rs.
//!
//! Handles `-e` (execute single statement) and `-f` (execute file) modes.
//! Both modes execute CQL statements, print results, and exit.

use std::io::{self, Write};

use anyhow::{Context, Result};

use crate::formatter;
use crate::session::CqlSession;

/// Execute a single CQL statement (`-e` mode) and print the result.
pub async fn execute_statement(session: &mut CqlSession, statement: &str) -> Result<()> {
    let mut stdout = io::stdout();

    for query in split_statements(statement) {
        let query = query.trim();
        if query.is_empty() {
            continue;
        }

        let result = session
            .execute(query)
            .await
            .with_context(|| format!("executing: {query}"))?;

        formatter::print_warnings(&mut stdout, &result)?;
        formatter::print_result(&mut stdout, &result)?;
        stdout.flush()?;
    }

    Ok(())
}

/// Execute CQL statements from a file (`-f` mode).
pub async fn execute_file(session: &mut CqlSession, path: &str) -> Result<()> {
    let contents =
        std::fs::read_to_string(path).with_context(|| format!("reading CQL file: {path}"))?;

    let mut stdout = io::stdout();

    for query in split_statements(&contents) {
        let query = query.trim();
        if query.is_empty() {
            continue;
        }

        // Skip comment-only lines
        if query.starts_with("--") || query.starts_with("//") {
            continue;
        }

        let result = session
            .execute(query)
            .await
            .with_context(|| format!("executing: {query}"))?;

        formatter::print_warnings(&mut stdout, &result)?;
        formatter::print_result(&mut stdout, &result)?;
        stdout.flush()?;
    }

    Ok(())
}

/// Split input text into individual CQL statements by semicolons.
///
/// Handles basic cases: ignores semicolons inside single-quoted strings.
/// A full parser (SP4) will handle all edge cases later.
fn split_statements(input: &str) -> Vec<&str> {
    let mut statements = Vec::new();
    let mut start = 0;
    let mut in_single_quote = false;
    let mut chars = input.char_indices().peekable();

    while let Some((i, ch)) = chars.next() {
        match ch {
            '\'' if !in_single_quote => {
                in_single_quote = true;
            }
            '\'' if in_single_quote => {
                // Check for escaped quote ('')
                if chars.peek().map(|(_, c)| *c) == Some('\'') {
                    chars.next(); // skip escaped quote
                } else {
                    in_single_quote = false;
                }
            }
            ';' if !in_single_quote => {
                let stmt = &input[start..i];
                if !stmt.trim().is_empty() {
                    statements.push(stmt.trim());
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    // Handle trailing statement without semicolon
    let remainder = &input[start..];
    if !remainder.trim().is_empty() {
        statements.push(remainder.trim());
    }

    statements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_single_statement() {
        let result = split_statements("SELECT * FROM users");
        assert_eq!(result, vec!["SELECT * FROM users"]);
    }

    #[test]
    fn split_single_statement_with_semicolon() {
        let result = split_statements("SELECT * FROM users;");
        assert_eq!(result, vec!["SELECT * FROM users"]);
    }

    #[test]
    fn split_multiple_statements() {
        let result = split_statements("SELECT 1; SELECT 2; SELECT 3");
        assert_eq!(result, vec!["SELECT 1", "SELECT 2", "SELECT 3"]);
    }

    #[test]
    fn split_ignores_empty() {
        let result = split_statements("SELECT 1; ; ; SELECT 2");
        assert_eq!(result, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn split_preserves_semicolon_in_string() {
        let result = split_statements("INSERT INTO t (a) VALUES ('hello;world')");
        assert_eq!(result, vec!["INSERT INTO t (a) VALUES ('hello;world')"]);
    }

    #[test]
    fn split_handles_escaped_quotes() {
        let result = split_statements("INSERT INTO t (a) VALUES ('it''s'); SELECT 1");
        assert_eq!(
            result,
            vec!["INSERT INTO t (a) VALUES ('it''s')", "SELECT 1"]
        );
    }

    #[test]
    fn split_empty_input() {
        let result = split_statements("");
        assert!(result.is_empty());
    }

    #[test]
    fn split_whitespace_only() {
        let result = split_statements("   \n\t  ");
        assert!(result.is_empty());
    }

    #[test]
    fn split_multiline_statements() {
        let input = "SELECT *\nFROM users\nWHERE id = 1;\nSELECT * FROM other";
        let result = split_statements(input);
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("SELECT *"));
        assert!(result[0].contains("WHERE id = 1"));
        assert!(result[1].contains("SELECT * FROM other"));
    }

    #[test]
    fn split_trailing_semicolons() {
        let result = split_statements("SELECT 1;;;");
        assert_eq!(result, vec!["SELECT 1"]);
    }
}

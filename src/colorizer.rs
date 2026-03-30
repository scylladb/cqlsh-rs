//! CQL syntax colorization for the REPL prompt and output.
//!
//! Uses the unified CQL lexer (`cql_lexer`) for context-aware tokenization,
//! which correctly distinguishes keywords from identifiers based on grammar
//! position (e.g., USERS after FROM is an identifier, not a keyword).
//! Also provides output coloring for query result values, headers, and errors
//! matching Python cqlsh's color scheme.

use crossterm::style::Stylize;

use crate::cql_lexer::{self, TokenKind};
use crate::driver::types::CqlValue;

/// CQL syntax colorizer using ANSI escape codes.
pub struct CqlColorizer {
    enabled: bool,
}

impl CqlColorizer {
    /// Create a new colorizer. If `enabled` is false, all methods return input unchanged.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Returns whether colorization is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Colorize a CQL result value matching Python cqlsh's color scheme.
    ///
    /// Color mapping:
    /// - Text/Ascii → yellow bold
    /// - Numeric/boolean/uuid/timestamp/date/time/duration/inet → green bold
    /// - Blob → dark magenta (non-bold)
    /// - Null → red bold
    /// - Collection delimiters → blue bold, inner values colored by type
    pub fn colorize_value(&self, value: &CqlValue) -> String {
        if !self.enabled {
            return value.to_string();
        }
        self.colorize_value_inner(value)
    }

    /// Colorize a column header (magenta bold, matching Python cqlsh default).
    pub fn colorize_header(&self, name: &str) -> String {
        if !self.enabled {
            return name.to_string();
        }
        format!("{}", name.magenta().bold())
    }

    /// Colorize an error message (red bold, matching Python cqlsh).
    pub fn colorize_error(&self, msg: &str) -> String {
        if !self.enabled {
            return msg.to_string();
        }
        format!("{}", msg.red().bold())
    }

    /// Colorize a warning message (red bold, matching Python cqlsh).
    pub fn colorize_warning(&self, msg: &str) -> String {
        self.colorize_error(msg)
    }

    /// Colorize the "Tracing session:" label (magenta bold).
    pub fn colorize_trace_label(&self, label: &str) -> String {
        if !self.enabled {
            return label.to_string();
        }
        format!("{}", label.magenta().bold())
    }

    /// Colorize the cluster name in the welcome message (blue bold).
    pub fn colorize_cluster_name(&self, name: &str) -> String {
        if !self.enabled {
            return name.to_string();
        }
        format!("{}", name.blue().bold())
    }

    /// Inner recursive value colorizer.
    fn colorize_value_inner(&self, value: &CqlValue) -> String {
        match value {
            CqlValue::Ascii(s) | CqlValue::Text(s) => {
                format!("{}", s.as_str().yellow().bold())
            }
            CqlValue::Int(_)
            | CqlValue::BigInt(_)
            | CqlValue::SmallInt(_)
            | CqlValue::TinyInt(_)
            | CqlValue::Float(_)
            | CqlValue::Double(_)
            | CqlValue::Decimal(_)
            | CqlValue::Varint(_)
            | CqlValue::Counter(_)
            | CqlValue::Boolean(_)
            | CqlValue::Uuid(_)
            | CqlValue::TimeUuid(_)
            | CqlValue::Timestamp(_)
            | CqlValue::Date(_)
            | CqlValue::Time(_)
            | CqlValue::Duration { .. }
            | CqlValue::Inet(_) => {
                format!("{}", value.to_string().green().bold())
            }
            CqlValue::Blob(_) => {
                format!("{}", value.to_string().dark_magenta())
            }
            CqlValue::Null => {
                format!("{}", "null".red().bold())
            }
            CqlValue::Unset => {
                format!("{}", "<unset>".red().bold())
            }
            CqlValue::List(items) => {
                let mut result = format!("{}", "[".blue().bold());
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        result.push_str(&format!("{}", ", ".blue().bold()));
                    }
                    result.push_str(&self.colorize_collection_element(item));
                }
                result.push_str(&format!("{}", "]".blue().bold()));
                result
            }
            CqlValue::Set(items) => {
                let mut result = format!("{}", "{".blue().bold());
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        result.push_str(&format!("{}", ", ".blue().bold()));
                    }
                    result.push_str(&self.colorize_collection_element(item));
                }
                result.push_str(&format!("{}", "}".blue().bold()));
                result
            }
            CqlValue::Map(entries) => {
                let mut result = format!("{}", "{".blue().bold());
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        result.push_str(&format!("{}", ", ".blue().bold()));
                    }
                    result.push_str(&self.colorize_collection_element(k));
                    result.push_str(&format!("{}", ": ".blue().bold()));
                    result.push_str(&self.colorize_collection_element(v));
                }
                result.push_str(&format!("{}", "}".blue().bold()));
                result
            }
            CqlValue::Tuple(items) => {
                let mut result = format!("{}", "(".blue().bold());
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        result.push_str(&format!("{}", ", ".blue().bold()));
                    }
                    match item {
                        Some(v) => result.push_str(&self.colorize_collection_element(v)),
                        None => result.push_str(&format!("{}", "null".red().bold())),
                    }
                }
                result.push_str(&format!("{}", ")".blue().bold()));
                result
            }
            CqlValue::UserDefinedType { fields, .. } => {
                let mut result = format!("{}", "{".blue().bold());
                for (i, (name, val)) in fields.iter().enumerate() {
                    if i > 0 {
                        result.push_str(&format!("{}", ", ".blue().bold()));
                    }
                    // UDT field names are yellow (like text)
                    result.push_str(&format!("{}", name.as_str().yellow().bold()));
                    result.push_str(&format!("{}", ": ".blue().bold()));
                    match val {
                        Some(v) => result.push_str(&self.colorize_collection_element(v)),
                        None => result.push_str(&format!("{}", "null".red().bold())),
                    }
                }
                result.push_str(&format!("{}", "}".blue().bold()));
                result
            }
        }
    }

    /// Colorize an element inside a collection, quoting strings like Display does.
    fn colorize_collection_element(&self, value: &CqlValue) -> String {
        match value {
            CqlValue::Ascii(s) | CqlValue::Text(s) => {
                // Inside collections, strings are quoted: 'value'
                let quoted = format!("'{}'", s.replace('\'', "''"));
                format!("{}", quoted.yellow().bold())
            }
            other => self.colorize_value_inner(other),
        }
    }

    /// Colorize a line of CQL input for display.
    ///
    /// Uses the unified CQL lexer for context-aware tokenization. Colors:
    /// - CQL keywords → bold blue
    /// - String literals → green
    /// - Numbers → cyan
    /// - Comments → dark grey
    /// - Identifiers, operators, punctuation → default (no color)
    pub fn colorize_line(&self, line: &str) -> String {
        if !self.enabled {
            return line.to_string();
        }

        let tokens = cql_lexer::tokenize(line);
        let mut result = String::with_capacity(line.len() * 2);

        for token in &tokens {
            match token.kind {
                TokenKind::Keyword => {
                    result.push_str(&format!("{}", token.text.as_str().blue().bold()));
                }
                TokenKind::StringLiteral | TokenKind::DollarStringLiteral => {
                    result.push_str(&format!("{}", token.text.as_str().green()));
                }
                TokenKind::NumberLiteral => {
                    result.push_str(&format!("{}", token.text.as_str().cyan()));
                }
                TokenKind::BlobLiteral => {
                    result.push_str(&format!("{}", token.text.as_str().dark_magenta()));
                }
                TokenKind::UuidLiteral => {
                    result.push_str(&format!("{}", token.text.as_str().green()));
                }
                TokenKind::BooleanLiteral => {
                    result.push_str(&format!("{}", token.text.as_str().green().bold()));
                }
                TokenKind::LineComment | TokenKind::BlockComment => {
                    result.push_str(&format!("{}", token.text.as_str().dark_grey()));
                }
                _ => {
                    result.push_str(&token.text);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_are_highlighted() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_line("SELECT * FROM users");
        assert!(output.contains("\x1b["), "should contain ANSI escape codes");
        assert!(output.contains("SELECT"));
        assert!(output.contains("FROM"));
    }

    #[test]
    fn colorizer_disabled_returns_unchanged() {
        let c = CqlColorizer::new(false);
        let output = c.colorize_line("SELECT * FROM users");
        assert_eq!(output, "SELECT * FROM users");
    }

    #[test]
    fn string_literals_are_colored() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_line("INSERT INTO t (a) VALUES ('hello')");
        // 'hello' should be green (contains ANSI codes)
        assert!(output.contains("\x1b["));
        assert!(output.contains("hello"));
    }

    #[test]
    fn numbers_are_colored() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_line("SELECT * FROM t LIMIT 100");
        assert!(output.contains("100"));
    }

    #[test]
    fn comments_are_colored() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_line("SELECT 1 -- test comment");
        assert!(output.contains("test comment"));
    }

    #[test]
    fn non_keywords_are_not_highlighted() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_line("my_table");
        // "my_table" is not a keyword, should not have ANSI codes
        assert!(!output.contains("\x1b["));
    }

    #[test]
    fn mixed_case_keywords() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_line("select * from users");
        assert!(
            output.contains("\x1b["),
            "lowercase keywords should also be highlighted"
        );
    }

    #[test]
    fn identifiers_after_from_not_highlighted() {
        let c = CqlColorizer::new(true);
        // "users" after FROM should NOT be highlighted as a keyword
        let output = c.colorize_line("SELECT * FROM users");
        // Extract the "users" portion — it should not contain ANSI codes
        // The output is: <colored SELECT> <space> <*> <space> <colored FROM> <space> <users>
        assert!(output.ends_with("users"));
    }

    #[test]
    fn keyword_names_as_identifiers_not_highlighted() {
        let c = CqlColorizer::new(true);
        // KEY and SET after FROM should be identifiers, not keywords
        let output_key = c.colorize_line("SELECT * FROM KEY");
        assert!(output_key.ends_with("KEY"), "KEY after FROM should not be highlighted");
        let output_set = c.colorize_line("SELECT * FROM SET");
        assert!(output_set.ends_with("SET"), "SET after FROM should not be highlighted");
    }

    #[test]
    fn qualified_name_after_dot_not_highlighted() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_line("SELECT * FROM ks.users");
        // "users" after dot should not be highlighted
        assert!(output.ends_with("users"));
    }

    // --- Output coloring tests ---

    #[test]
    fn colorize_text_value_yellow() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_value(&CqlValue::Text("hello".to_string()));
        assert!(output.contains("\x1b["), "should contain ANSI codes");
        assert!(output.contains("hello"));
    }

    #[test]
    fn colorize_int_value_green() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_value(&CqlValue::Int(42));
        assert!(output.contains("\x1b["), "should contain ANSI codes");
        assert!(output.contains("42"));
    }

    #[test]
    fn colorize_null_value_red() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_value(&CqlValue::Null);
        assert!(output.contains("\x1b["), "should contain ANSI codes");
        assert!(output.contains("null"));
    }

    #[test]
    fn colorize_blob_value_dark_magenta() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_value(&CqlValue::Blob(vec![0xde, 0xad]));
        assert!(output.contains("\x1b["), "should contain ANSI codes");
        assert!(output.contains("dead"));
    }

    #[test]
    fn colorize_list_with_blue_delimiters() {
        let c = CqlColorizer::new(true);
        let list = CqlValue::List(vec![CqlValue::Int(1), CqlValue::Int(2)]);
        let output = c.colorize_value(&list);
        assert!(output.contains("\x1b["), "should contain ANSI codes");
    }

    #[test]
    fn colorize_value_disabled_returns_plain() {
        let c = CqlColorizer::new(false);
        let output = c.colorize_value(&CqlValue::Text("hello".to_string()));
        assert_eq!(output, "hello");
    }

    #[test]
    fn colorize_header_magenta() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_header("name");
        assert!(output.contains("\x1b["), "should contain ANSI codes");
        assert!(output.contains("name"));
    }

    #[test]
    fn colorize_error_red() {
        let c = CqlColorizer::new(true);
        let output = c.colorize_error("SyntaxException: bad input");
        assert!(output.contains("\x1b["), "should contain ANSI codes");
        assert!(output.contains("SyntaxException"));
    }

    #[test]
    fn colorize_map_with_colored_elements() {
        let c = CqlColorizer::new(true);
        let map = CqlValue::Map(vec![(CqlValue::Text("key".to_string()), CqlValue::Int(42))]);
        let output = c.colorize_value(&map);
        assert!(output.contains("\x1b["), "should contain ANSI codes");
    }

    #[test]
    fn colorize_udt_field_names_yellow() {
        let c = CqlColorizer::new(true);
        let udt = CqlValue::UserDefinedType {
            keyspace: "ks".to_string(),
            type_name: "my_type".to_string(),
            fields: vec![
                (
                    "name".to_string(),
                    Some(CqlValue::Text("Alice".to_string())),
                ),
                ("age".to_string(), Some(CqlValue::Int(30))),
            ],
        };
        let output = c.colorize_value(&udt);
        assert!(output.contains("\x1b["), "should contain ANSI codes");
    }
}

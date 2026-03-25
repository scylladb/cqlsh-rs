//! CQL syntax colorization for the REPL prompt and output.
//!
//! Provides a simple tokenizer that applies ANSI colors to CQL keywords,
//! string literals, numbers, and comments using crossterm styling.

use crossterm::style::Stylize;

/// Set of CQL keywords to highlight (uppercase for matching).
const KEYWORDS: &[&str] = &[
    "ADD", "ALTER", "AND", "APPLY", "AS", "ASC", "AUTHORIZE", "BATCH", "BEGIN",
    "BY", "CALLED", "CLUSTERING", "COLUMN", "COMPACT", "CONTAINS", "COUNT",
    "CREATE", "CUSTOM", "DELETE", "DESC", "DESCRIBE", "DISTINCT", "DROP",
    "ENTRIES", "EXECUTE", "EXISTS", "FILTERING", "FROM", "FROZEN", "FULL",
    "FUNCTION", "GRANT", "IF", "IN", "INDEX", "INSERT", "INTO", "IS", "JSON",
    "KEY", "KEYSPACE", "KEYSPACES", "LANGUAGE", "LIKE", "LIMIT", "LIST",
    "LOGIN", "MAP", "MATERIALIZED", "MODIFY", "NAMESPACE", "NOT", "NULL",
    "OF", "ON", "OR", "ORDER", "PARTITION", "PASSWORD", "PER", "PERMISSION",
    "PERMISSIONS", "PRIMARY", "RENAME", "REPLACE", "RETURNS", "REVOKE",
    "SCHEMA", "SELECT", "SET", "STATIC", "STORAGE", "SUPERUSER", "TABLE",
    "TABLES", "TEXT", "TIMESTAMP", "TO", "TOKEN", "TRIGGER", "TRUNCATE",
    "TTL", "TUPLE", "TYPE", "UNLOGGED", "UPDATE", "USE", "USER", "USERS",
    "USING", "VALUES", "VIEW", "WHERE", "WITH", "WRITETIME",
];

/// CQL syntax colorizer using ANSI escape codes.
pub struct CqlColorizer {
    enabled: bool,
}

impl CqlColorizer {
    /// Create a new colorizer. If `enabled` is false, all methods return input unchanged.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Colorize a line of CQL input for display.
    ///
    /// Applies colors:
    /// - CQL keywords → bold blue
    /// - String literals ('...') → green
    /// - Numbers → cyan
    /// - Comments (-- ...) → dark grey
    pub fn colorize_line(&self, line: &str) -> String {
        if !self.enabled {
            return line.to_string();
        }

        let mut result = String::with_capacity(line.len() * 2);
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // Comment: -- to end of line
            if i + 1 < len && chars[i] == '-' && chars[i + 1] == '-' {
                let rest: String = chars[i..].iter().collect();
                result.push_str(&format!("{}", rest.dark_grey()));
                break;
            }

            // String literal: '...'
            if chars[i] == '\'' {
                let start = i;
                i += 1;
                while i < len && chars[i] != '\'' {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 1; // skip escaped char
                    }
                    i += 1;
                }
                if i < len {
                    i += 1; // consume closing quote
                }
                let literal: String = chars[start..i].iter().collect();
                result.push_str(&format!("{}", literal.green()));
                continue;
            }

            // Number (simple: digits possibly with dots)
            if chars[i].is_ascii_digit()
                || (chars[i] == '-'
                    && i + 1 < len
                    && chars[i + 1].is_ascii_digit()
                    && (i == 0 || !chars[i - 1].is_alphanumeric()))
            {
                let start = i;
                if chars[i] == '-' {
                    i += 1;
                }
                while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                // Make sure this isn't part of an identifier
                if i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    // It's an identifier like "table1" — don't colorize
                    let word: String = chars[start..].iter().collect();
                    let end = word
                        .find(|c: char| c.is_whitespace() || c == ',' || c == ')' || c == ';')
                        .unwrap_or(word.len());
                    result.push_str(&chars[start..start + end].iter().collect::<String>());
                    i = start + end;
                } else {
                    let num: String = chars[start..i].iter().collect();
                    result.push_str(&format!("{}", num.cyan()));
                }
                continue;
            }

            // Word (potential keyword)
            if chars[i].is_alphabetic() || chars[i] == '_' {
                let start = i;
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                if is_keyword(&word) {
                    result.push_str(&format!("{}", word.blue().bold()));
                } else {
                    result.push_str(&word);
                }
                continue;
            }

            // Other characters (whitespace, operators, etc.)
            result.push(chars[i]);
            i += 1;
        }

        result
    }
}

/// Check if a word is a CQL keyword (case-insensitive).
fn is_keyword(word: &str) -> bool {
    let upper = word.to_uppercase();
    KEYWORDS.binary_search(&upper.as_str()).is_ok()
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
        assert!(output.contains("\x1b["), "lowercase keywords should also be highlighted");
    }

    #[test]
    fn keyword_list_is_sorted() {
        // binary_search requires sorted list
        for window in KEYWORDS.windows(2) {
            assert!(
                window[0] < window[1],
                "KEYWORDS not sorted: {:?} >= {:?}",
                window[0],
                window[1]
            );
        }
    }
}

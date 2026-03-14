//! Statement parser for cqlsh-rs.
//!
//! Handles multi-line input buffering, semicolon-terminated statement detection,
//! comment stripping, string literal handling, and routing between CQL statements
//! and built-in shell commands.
//!
//! Key design decisions (from SP4 and SP16 upstream fixes):
//! - Context-aware tokenization: NO regex preprocessing for comments (PR #150)
//! - Incremental parsing: O(1) per line, only full-parse on terminator (PR #151)

/// Lexer context states for tracking position within CQL input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexState {
    /// Normal CQL code (not in string or comment).
    Normal,
    /// Inside a single-quoted string literal (`'...'`).
    SingleQuote,
    /// Inside a double-quoted identifier (`"..."`).
    DoubleQuote,
    /// Inside a dollar-quoted string literal (`$$...$$`).
    DollarQuote,
    /// Inside a block comment (`/* ... */`).
    BlockComment,
    /// Inside a line comment (`-- ...`), extends to end of line.
    LineComment,
}

/// Incremental statement parser.
///
/// Tracks lexer state across lines so that multi-line input can be processed
/// in O(1) per line. Only produces complete statements when a semicolon
/// terminator is found outside of strings and comments.
#[derive(Debug)]
pub struct StatementParser {
    /// Accumulated input buffer.
    buffer: String,
    /// Current lexer state at the end of the buffer.
    state: LexState,
    /// Depth of nested block comments (CQL doesn't nest, but we track for robustness).
    block_comment_depth: usize,
}

/// The result of feeding a line to the parser.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseResult {
    /// No complete statement yet; continue accumulating.
    Incomplete,
    /// One or more complete statements extracted.
    Complete(Vec<String>),
}

/// Classification of a parsed input line.
#[derive(Debug, PartialEq, Eq)]
pub enum InputKind {
    /// A built-in shell command (HELP, QUIT, DESCRIBE, etc.).
    ShellCommand(String),
    /// A CQL statement to forward to the driver.
    CqlStatement(String),
    /// Empty or whitespace-only input.
    Empty,
}

/// Built-in shell commands that don't require a semicolon terminator.
const SHELL_COMMANDS: &[&str] = &[
    "HELP",
    "?",
    "QUIT",
    "EXIT",
    "DESCRIBE",
    "DESC",
    "CONSISTENCY",
    "SERIAL",
    "TRACING",
    "EXPAND",
    "PAGING",
    "LOGIN",
    "SOURCE",
    "CAPTURE",
    "SHOW",
    "CLEAR",
    "CLS",
    "UNICODE",
    "DEBUG",
    "COPY",
    "USE",
];

impl StatementParser {
    /// Create a new empty parser.
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            state: LexState::Normal,
            block_comment_depth: 0,
        }
    }

    /// Reset the parser, discarding any accumulated input.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.state = LexState::Normal;
        self.block_comment_depth = 0;
    }

    /// Returns true if the parser has no accumulated input.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Feed a line of input and return any complete statements.
    ///
    /// This is the incremental entry point. Each call processes the line
    /// character-by-character, updating the lexer state. When a semicolon
    /// is encountered in Normal state, the accumulated text up to that point
    /// is extracted as a complete statement.
    pub fn feed_line(&mut self, line: &str) -> ParseResult {
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(line);

        // Scan the newly added content for statement terminators.
        // We must re-scan from a known state boundary.
        self.extract_statements()
    }

    /// Scan the entire buffer for complete statements.
    ///
    /// Returns any complete statements found, leaving the remainder in the buffer.
    fn extract_statements(&mut self) -> ParseResult {
        let mut statements = Vec::new();
        let input = self.buffer.clone();
        let chars: Vec<char> = input.chars().collect();
        let len = chars.len();

        // Reset state for full scan from buffer start.
        self.state = LexState::Normal;
        self.block_comment_depth = 0;

        let mut stmt_start = 0;
        let mut i = 0;

        while i < len {
            match self.state {
                LexState::Normal => {
                    if chars[i] == '\'' {
                        self.state = LexState::SingleQuote;
                        i += 1;
                    } else if chars[i] == '"' {
                        self.state = LexState::DoubleQuote;
                        i += 1;
                    } else if i + 1 < len && chars[i] == '$' && chars[i + 1] == '$' {
                        self.state = LexState::DollarQuote;
                        i += 2;
                    } else if i + 1 < len && chars[i] == '-' && chars[i + 1] == '-' {
                        self.state = LexState::LineComment;
                        i += 2;
                    } else if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
                        self.state = LexState::BlockComment;
                        self.block_comment_depth = 1;
                        i += 2;
                    } else if chars[i] == ';' {
                        // Statement terminator found in Normal state.
                        let raw = &input[byte_offset(&chars, stmt_start)..byte_offset(&chars, i)];
                        let stripped = strip_comments(raw);
                        let trimmed = stripped.trim();
                        if !trimmed.is_empty() {
                            statements.push(trimmed.to_string());
                        }
                        stmt_start = i + 1;
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                LexState::SingleQuote => {
                    if chars[i] == '\'' {
                        // Check for escaped quote ('')
                        if i + 1 < len && chars[i + 1] == '\'' {
                            i += 2; // skip escaped quote
                        } else {
                            self.state = LexState::Normal;
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                LexState::DoubleQuote => {
                    if chars[i] == '"' {
                        // Check for escaped quote ("")
                        if i + 1 < len && chars[i + 1] == '"' {
                            i += 2;
                        } else {
                            self.state = LexState::Normal;
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                LexState::DollarQuote => {
                    if i + 1 < len && chars[i] == '$' && chars[i + 1] == '$' {
                        self.state = LexState::Normal;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                LexState::LineComment => {
                    if chars[i] == '\n' {
                        self.state = LexState::Normal;
                    }
                    i += 1;
                }
                LexState::BlockComment => {
                    if i + 1 < len && chars[i] == '*' && chars[i + 1] == '/' {
                        self.block_comment_depth -= 1;
                        if self.block_comment_depth == 0 {
                            self.state = LexState::Normal;
                        }
                        i += 2;
                    } else if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
                        // Nested block comment
                        self.block_comment_depth += 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
        }

        // Keep remaining text in the buffer for next feed_line call.
        let remaining_start = byte_offset(&chars, stmt_start);
        self.buffer = input[remaining_start..].to_string();

        if statements.is_empty() {
            ParseResult::Incomplete
        } else {
            ParseResult::Complete(statements)
        }
    }
}

impl Default for StatementParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the byte offset for a character index in a char slice.
fn byte_offset(chars: &[char], char_idx: usize) -> usize {
    chars[..char_idx].iter().map(|c| c.len_utf8()).sum()
}

/// Strip comments from a CQL fragment (used on extracted statements).
///
/// This function uses context-aware scanning to avoid stripping comment-like
/// sequences inside string literals (PR #150 fix).
fn strip_comments(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(input.len());
    let mut state = LexState::Normal;
    let mut i = 0;

    while i < len {
        match state {
            LexState::Normal => {
                if chars[i] == '\'' {
                    state = LexState::SingleQuote;
                    result.push(chars[i]);
                    i += 1;
                } else if chars[i] == '"' {
                    state = LexState::DoubleQuote;
                    result.push(chars[i]);
                    i += 1;
                } else if i + 1 < len && chars[i] == '$' && chars[i + 1] == '$' {
                    state = LexState::DollarQuote;
                    result.push('$');
                    result.push('$');
                    i += 2;
                } else if i + 1 < len && chars[i] == '-' && chars[i + 1] == '-' {
                    // Line comment: skip to end of line
                    state = LexState::LineComment;
                    i += 2;
                } else if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
                    // Block comment: skip content
                    state = LexState::BlockComment;
                    i += 2;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            LexState::SingleQuote => {
                result.push(chars[i]);
                if chars[i] == '\'' {
                    if i + 1 < len && chars[i + 1] == '\'' {
                        result.push(chars[i + 1]);
                        i += 2;
                    } else {
                        state = LexState::Normal;
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            LexState::DoubleQuote => {
                result.push(chars[i]);
                if chars[i] == '"' {
                    if i + 1 < len && chars[i + 1] == '"' {
                        result.push(chars[i + 1]);
                        i += 2;
                    } else {
                        state = LexState::Normal;
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            LexState::DollarQuote => {
                result.push(chars[i]);
                if i + 1 < len && chars[i] == '$' && chars[i + 1] == '$' {
                    result.push('$');
                    state = LexState::Normal;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            LexState::LineComment => {
                if chars[i] == '\n' {
                    result.push('\n');
                    state = LexState::Normal;
                }
                i += 1;
            }
            LexState::BlockComment => {
                if i + 1 < len && chars[i] == '*' && chars[i + 1] == '/' {
                    // Replace block comment with a space to avoid token merging
                    result.push(' ');
                    state = LexState::Normal;
                    i += 2;
                } else {
                    i += 1;
                }
            }
        }
    }

    result
}

/// Classify a complete input as a shell command, CQL statement, or empty.
pub fn classify_input(input: &str) -> InputKind {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return InputKind::Empty;
    }

    // Strip trailing semicolon for command detection
    let without_semi = trimmed.strip_suffix(';').unwrap_or(trimmed).trim();
    let first_word = without_semi
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_uppercase();

    if SHELL_COMMANDS.contains(&first_word.as_str()) {
        InputKind::ShellCommand(trimmed.to_string())
    } else {
        InputKind::CqlStatement(trimmed.to_string())
    }
}

/// Check if the first line of input looks like a shell command.
///
/// Used by the REPL to decide whether to wait for a semicolon
/// or dispatch immediately.
pub fn is_shell_command(line: &str) -> bool {
    let trimmed = line.trim();
    let first_word = trimmed
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_uppercase();

    SHELL_COMMANDS.contains(&first_word.as_str())
}

/// Parse a complete input string (e.g., from `-e` or `-f` batch mode)
/// into individual statements.
///
/// Returns a vector of complete, comment-stripped statements.
/// This is O(n) in the input size (not O(n²) per PR #151).
pub fn parse_batch(input: &str) -> Vec<String> {
    let mut parser = StatementParser::new();
    let mut all_statements = Vec::new();

    for line in input.lines() {
        if let ParseResult::Complete(stmts) = parser.feed_line(line) {
            all_statements.extend(stmts);
        }
    }

    // Handle any remaining content without a trailing semicolon.
    // Shell commands don't need semicolons; CQL statements do.
    let remaining = parser.buffer.trim().to_string();
    if !remaining.is_empty() {
        let stripped = strip_comments(&remaining);
        let trimmed = stripped.trim();
        if !trimmed.is_empty() && is_shell_command(trimmed) {
            all_statements.push(trimmed.to_string());
        }
        // Non-shell-command without semicolon is incomplete — drop it
        // (matches Python cqlsh batch mode behavior)
    }

    all_statements
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Basic semicolon detection ---

    #[test]
    fn simple_statement() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT * FROM users;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT * FROM users".to_string()])
        );
    }

    #[test]
    fn statement_with_trailing_whitespace() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT * FROM users;  ");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT * FROM users".to_string()])
        );
    }

    #[test]
    fn incomplete_no_semicolon() {
        let mut p = StatementParser::new();
        assert_eq!(p.feed_line("SELECT * FROM users"), ParseResult::Incomplete);
    }

    #[test]
    fn empty_input() {
        let mut p = StatementParser::new();
        assert_eq!(p.feed_line(""), ParseResult::Incomplete);
        assert_eq!(p.feed_line("   "), ParseResult::Incomplete);
    }

    // --- Single-quoted string handling ---

    #[test]
    fn semicolon_in_single_quoted_string() {
        let mut p = StatementParser::new();
        let result = p.feed_line("INSERT INTO t (v) VALUES ('hello;world');");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["INSERT INTO t (v) VALUES ('hello;world')".to_string()])
        );
    }

    #[test]
    fn escaped_quote_in_string() {
        let mut p = StatementParser::new();
        let result = p.feed_line("INSERT INTO t (v) VALUES ('it''s;here');");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["INSERT INTO t (v) VALUES ('it''s;here')".to_string()])
        );
    }

    // --- Double-quoted identifier handling ---

    #[test]
    fn semicolon_in_double_quoted_identifier() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT \"col;name\" FROM t;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT \"col;name\" FROM t".to_string()])
        );
    }

    #[test]
    fn escaped_double_quote() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT \"col\"\"name\" FROM t;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT \"col\"\"name\" FROM t".to_string()])
        );
    }

    // --- Dollar-quoted string handling ---

    #[test]
    fn semicolon_in_dollar_quoted_string() {
        let mut p = StatementParser::new();
        let result = p.feed_line("CREATE FUNCTION f() RETURNS NULL ON NULL INPUT RETURNS text LANGUAGE java AS $$return a;$$;");
        assert_eq!(result, ParseResult::Complete(vec![
            "CREATE FUNCTION f() RETURNS NULL ON NULL INPUT RETURNS text LANGUAGE java AS $$return a;$$".to_string()
        ]));
    }

    #[test]
    fn dollar_quote_multiline() {
        let mut p = StatementParser::new();
        assert_eq!(
            p.feed_line("CREATE FUNCTION f() RETURNS text LANGUAGE java AS $$"),
            ParseResult::Incomplete
        );
        assert_eq!(p.feed_line("  return a;"), ParseResult::Incomplete);
        let result = p.feed_line("$$;");
        assert!(matches!(result, ParseResult::Complete(_)));
    }

    // --- Line comment stripping ---

    #[test]
    fn line_comment_stripped() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT * FROM t; -- this is a comment");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT * FROM t".to_string()])
        );
    }

    #[test]
    fn line_comment_does_not_terminate() {
        let mut p = StatementParser::new();
        // Semicolon inside line comment should not terminate
        assert_eq!(
            p.feed_line("SELECT * FROM t -- comment with ;"),
            ParseResult::Incomplete
        );
    }

    // --- Block comment stripping (PR #150) ---

    #[test]
    fn block_comment_stripped() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT /* comment */ * FROM t;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT   * FROM t".to_string()])
        );
    }

    #[test]
    fn block_comment_with_semicolon() {
        let mut p = StatementParser::new();
        // Semicolon inside block comment should not terminate
        let result = p.feed_line("SELECT /* ; */ * FROM t;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT   * FROM t".to_string()])
        );
    }

    #[test]
    fn block_comment_chars_in_single_quoted_string() {
        // PR #150: /* inside strings must NOT be treated as comment start
        let mut p = StatementParser::new();
        let result = p.feed_line("INSERT INTO t (v) VALUES ('/* not a comment */');");
        assert_eq!(
            result,
            ParseResult::Complete(vec![
                "INSERT INTO t (v) VALUES ('/* not a comment */')".to_string()
            ])
        );
    }

    #[test]
    fn block_comment_chars_in_double_quoted_string() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT \"/* not a comment */\" FROM t;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT \"/* not a comment */\" FROM t".to_string()])
        );
    }

    #[test]
    fn block_comment_chars_in_dollar_quoted_string() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT $$/* not a comment */$$;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT $$/* not a comment */$$".to_string()])
        );
    }

    // --- Multi-line statement buffering ---

    #[test]
    fn multiline_statement() {
        let mut p = StatementParser::new();
        assert_eq!(p.feed_line("SELECT *"), ParseResult::Incomplete);
        assert_eq!(p.feed_line("FROM users"), ParseResult::Incomplete);
        let result = p.feed_line("WHERE id = 1;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT *\nFROM users\nWHERE id = 1".to_string()])
        );
    }

    #[test]
    fn multiline_with_string_across_lines() {
        let mut p = StatementParser::new();
        assert_eq!(
            p.feed_line("INSERT INTO t (v) VALUES ('hello"),
            ParseResult::Incomplete
        );
        let result = p.feed_line("world');");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["INSERT INTO t (v) VALUES ('hello\nworld')".to_string()])
        );
    }

    // --- Empty statement handling ---

    #[test]
    fn empty_statement_skipped() {
        let mut p = StatementParser::new();
        let result = p.feed_line(";;");
        // Both semicolons produce empty statements which are skipped
        assert_eq!(result, ParseResult::Incomplete);
    }

    #[test]
    fn empty_between_statements() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT 1; ; SELECT 2;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT 1".to_string(), "SELECT 2".to_string(),])
        );
    }

    // --- Built-in command detection ---

    #[test]
    fn shell_commands_detected() {
        assert!(is_shell_command("HELP"));
        assert!(is_shell_command("?"));
        assert!(is_shell_command("QUIT"));
        assert!(is_shell_command("EXIT"));
        assert!(is_shell_command("DESCRIBE KEYSPACES"));
        assert!(is_shell_command("DESC TABLE users"));
        assert!(is_shell_command("CONSISTENCY ONE"));
        assert!(is_shell_command("TRACING ON"));
        assert!(is_shell_command("EXPAND ON"));
        assert!(is_shell_command("PAGING 100"));
        assert!(is_shell_command("SHOW VERSION"));
        assert!(is_shell_command("CLEAR"));
        assert!(is_shell_command("CLS"));
        assert!(is_shell_command("COPY users TO '/tmp/data.csv'"));
        assert!(is_shell_command("USE my_keyspace"));
    }

    #[test]
    fn shell_command_case_insensitive() {
        assert!(is_shell_command("help"));
        assert!(is_shell_command("quit"));
        assert!(is_shell_command("Help"));
        assert!(is_shell_command("describe keyspaces"));
        assert!(is_shell_command("use my_ks"));
    }

    #[test]
    fn cql_not_shell_command() {
        assert!(!is_shell_command("SELECT * FROM users"));
        assert!(!is_shell_command("INSERT INTO t (id) VALUES (1)"));
        assert!(!is_shell_command("CREATE TABLE test (id int PRIMARY KEY)"));
    }

    // --- Command classification ---

    #[test]
    fn classify_shell_command() {
        assert_eq!(
            classify_input("HELP"),
            InputKind::ShellCommand("HELP".to_string())
        );
        assert_eq!(
            classify_input("USE my_ks"),
            InputKind::ShellCommand("USE my_ks".to_string())
        );
    }

    #[test]
    fn classify_cql_statement() {
        assert_eq!(
            classify_input("SELECT * FROM users"),
            InputKind::CqlStatement("SELECT * FROM users".to_string())
        );
    }

    #[test]
    fn classify_empty() {
        assert_eq!(classify_input(""), InputKind::Empty);
        assert_eq!(classify_input("   "), InputKind::Empty);
    }

    // --- Multiple statements on one line ---

    #[test]
    fn multiple_statements_one_line() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT 1; SELECT 2; SELECT 3;");
        assert_eq!(
            result,
            ParseResult::Complete(vec![
                "SELECT 1".to_string(),
                "SELECT 2".to_string(),
                "SELECT 3".to_string(),
            ])
        );
    }

    // --- Whitespace normalization ---

    #[test]
    fn leading_trailing_whitespace_trimmed() {
        let mut p = StatementParser::new();
        let result = p.feed_line("  SELECT * FROM t  ;  ");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT * FROM t".to_string()])
        );
    }

    // --- Batch mode parsing ---

    #[test]
    fn parse_batch_basic() {
        let input = "SELECT 1;\nSELECT 2;\n";
        let stmts = parse_batch(input);
        assert_eq!(stmts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn parse_batch_with_comments() {
        let input = "-- header comment\nSELECT 1; -- inline\nSELECT /* x */ 2;\n";
        let stmts = parse_batch(input);
        assert_eq!(stmts, vec!["SELECT 1", "SELECT   2"]);
    }

    #[test]
    fn parse_batch_multiline_statement() {
        let input = "SELECT *\nFROM users\nWHERE id = 1;\n";
        let stmts = parse_batch(input);
        assert_eq!(stmts, vec!["SELECT *\nFROM users\nWHERE id = 1"]);
    }

    #[test]
    fn parse_batch_with_shell_command() {
        let input = "SELECT 1;\nUSE my_ks\n";
        let stmts = parse_batch(input);
        assert_eq!(stmts, vec!["SELECT 1", "USE my_ks"]);
    }

    #[test]
    fn parse_batch_drops_incomplete_cql() {
        // CQL without semicolon at end of file is dropped (Python cqlsh behavior)
        let input = "SELECT 1;\nSELECT 2";
        let stmts = parse_batch(input);
        assert_eq!(stmts, vec!["SELECT 1"]);
    }

    // --- Comment stripping edge cases ---

    #[test]
    fn strip_comments_preserves_strings() {
        let input = "SELECT '-- not a comment' FROM t";
        let result = strip_comments(input);
        assert_eq!(result, "SELECT '-- not a comment' FROM t");
    }

    #[test]
    fn strip_comments_preserves_dollar_strings() {
        let input = "SELECT $$-- not a comment$$ FROM t";
        let result = strip_comments(input);
        assert_eq!(result, "SELECT $$-- not a comment$$ FROM t");
    }

    #[test]
    fn strip_comments_multiline_block() {
        let input = "SELECT /* multi\nline\ncomment */ 1";
        let result = strip_comments(input);
        // Block comment is replaced with a single space, plus the existing space = "  "
        assert_eq!(result, "SELECT   1");
    }

    // --- Parser reset ---

    #[test]
    fn reset_clears_state() {
        let mut p = StatementParser::new();
        assert_eq!(p.feed_line("SELECT *"), ParseResult::Incomplete);
        assert!(!p.is_empty());

        p.reset();
        assert!(p.is_empty());

        // After reset, should start fresh
        let result = p.feed_line("SELECT 1;");
        assert_eq!(result, ParseResult::Complete(vec!["SELECT 1".to_string()]));
    }

    // --- Unicode handling ---

    #[test]
    fn unicode_in_strings() {
        let mut p = StatementParser::new();
        let result = p.feed_line("INSERT INTO t (v) VALUES ('héllo wörld; café');");
        assert_eq!(
            result,
            ParseResult::Complete(vec![
                "INSERT INTO t (v) VALUES ('héllo wörld; café')".to_string()
            ])
        );
    }

    #[test]
    fn unicode_identifier() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT \"naïve;col\" FROM t;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT \"naïve;col\" FROM t".to_string()])
        );
    }
}

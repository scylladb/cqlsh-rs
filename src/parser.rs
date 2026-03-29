//! Statement parser for cqlsh-rs.
//!
//! Handles multi-line input buffering, semicolon-terminated statement detection,
//! comment stripping, string literal handling, and routing between CQL statements
//! and built-in shell commands.
//!
//! Key design decisions (from SP4 and SP16 upstream fixes):
//! - Context-aware tokenization: NO regex preprocessing for comments (PR #150)
//! - Truly incremental parsing: O(n) total work via scan_offset tracking (PR #151)

/// Lexer context states for tracking position within CQL input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum LexState {
    /// Normal CQL code (not in string or comment).
    #[default]
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
/// Tracks lexer state across `feed_line` calls so that each call only scans
/// the newly appended bytes. Total work is O(n) over the lifetime of the parser,
/// not O(n²). See PR #151 for why this matters.
#[derive(Debug, Default)]
pub struct StatementParser {
    /// Accumulated input buffer.
    buffer: String,
    /// Byte offset in `buffer` where the next scan should resume.
    scan_offset: usize,
    /// Byte offset of the start of the current (in-progress) statement.
    stmt_start: usize,
    /// Current lexer state at `scan_offset`.
    state: LexState,
    /// Depth of nested block comments.
    block_comment_depth: usize,
    /// True when we are inside a `BEGIN BATCH … APPLY BATCH` block.
    /// Semicolons inside a batch do not terminate the batch statement.
    in_batch: bool,
}

/// The result of feeding a line to the parser.
#[derive(Debug, PartialEq, Eq)]
#[must_use]
pub enum ParseResult {
    /// No complete statement yet; continue accumulating.
    Incomplete,
    /// One or more complete statements extracted.
    Complete(Vec<String>),
}

/// Classification of a parsed input line.
#[derive(Debug, PartialEq, Eq)]
#[must_use]
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the parser, discarding any accumulated input.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.scan_offset = 0;
        self.stmt_start = 0;
        self.state = LexState::Normal;
        self.block_comment_depth = 0;
        self.in_batch = false;
    }

    /// Returns true if the parser has no accumulated input.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the remaining unparsed content in the buffer.
    #[must_use]
    pub fn remaining(&self) -> &str {
        &self.buffer[self.stmt_start..]
    }

    /// Feed a line of input and return any complete statements.
    ///
    /// This is the incremental entry point. Each call scans only the newly
    /// appended bytes, preserving lexer state from the previous call.
    /// Total work across all `feed_line` calls is O(n).
    pub fn feed_line(&mut self, line: &str) -> ParseResult {
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(line);

        self.scan_for_statements()
    }

    /// Scan from `scan_offset` forward for statement terminators.
    ///
    /// Only scans newly appended bytes — does NOT re-scan from the start.
    /// State (`self.state`, `self.block_comment_depth`) is preserved across calls.
    fn scan_for_statements(&mut self) -> ParseResult {
        let mut statements = Vec::new();

        // We work on byte offsets using char_indices over the unscanned portion.
        // But we need to handle multi-byte chars correctly, so iterate chars.
        let buf = self.buffer.as_bytes();
        let len = buf.len();
        let mut i = self.scan_offset;

        while i < len {
            let (ch, char_len) = decode_char_at(&self.buffer, i);

            match self.state {
                LexState::Normal => {
                    if ch == '\'' {
                        self.state = LexState::SingleQuote;
                        i += char_len;
                    } else if ch == '"' {
                        self.state = LexState::DoubleQuote;
                        i += char_len;
                    } else if ch == '$' && i + 1 < len && self.buffer.as_bytes()[i + 1] == b'$' {
                        self.state = LexState::DollarQuote;
                        i += 2;
                    } else if ch == '-' && i + 1 < len && self.buffer.as_bytes()[i + 1] == b'-' {
                        self.state = LexState::LineComment;
                        i += 2;
                    } else if ch == '/' && i + 1 < len && self.buffer.as_bytes()[i + 1] == b'*' {
                        self.state = LexState::BlockComment;
                        self.block_comment_depth = 1;
                        i += 2;
                    } else if ch == ';' {
                        // Statement terminator found in Normal state.
                        let raw = &self.buffer[self.stmt_start..i];
                        let stripped = strip_comments(raw);
                        let trimmed = stripped.trim();

                        if self.in_batch {
                            // Inside BEGIN BATCH … APPLY BATCH: semicolons
                            // between DML statements are part of the batch
                            // syntax, not statement terminators.  Only emit
                            // when APPLY BATCH has been reached.
                            if ends_with_apply_batch(trimmed) {
                                self.in_batch = false;
                                if !trimmed.is_empty() {
                                    statements.push(trimmed.to_string());
                                }
                                self.stmt_start = i + 1;
                            }
                            // Otherwise keep accumulating; do NOT advance stmt_start.
                            i += 1;
                        } else if starts_with_begin_batch(trimmed) {
                            // Opening of a BATCH block: treat the ';' as
                            // internal to the batch, not as a terminator.
                            self.in_batch = true;
                            // Do NOT advance stmt_start — keep accumulating
                            // from the start of BEGIN BATCH.
                            i += 1;
                        } else {
                            if !trimmed.is_empty() {
                                statements.push(trimmed.to_string());
                            }
                            self.stmt_start = i + 1; // skip the ';'
                            i += 1;
                        }
                    } else {
                        i += char_len;
                    }
                }
                LexState::SingleQuote => {
                    if ch == '\'' {
                        // Check for escaped quote ('')
                        if i + 1 < len && self.buffer.as_bytes()[i + 1] == b'\'' {
                            i += 2; // skip escaped quote
                        } else {
                            self.state = LexState::Normal;
                            i += 1;
                        }
                    } else {
                        i += char_len;
                    }
                }
                LexState::DoubleQuote => {
                    if ch == '"' {
                        // Check for escaped quote ("")
                        if i + 1 < len && self.buffer.as_bytes()[i + 1] == b'"' {
                            i += 2;
                        } else {
                            self.state = LexState::Normal;
                            i += 1;
                        }
                    } else {
                        i += char_len;
                    }
                }
                LexState::DollarQuote => {
                    if ch == '$' && i + 1 < len && self.buffer.as_bytes()[i + 1] == b'$' {
                        self.state = LexState::Normal;
                        i += 2;
                    } else {
                        i += char_len;
                    }
                }
                LexState::LineComment => {
                    if ch == '\n' {
                        self.state = LexState::Normal;
                    }
                    i += char_len;
                }
                LexState::BlockComment => {
                    if ch == '*' && i + 1 < len && self.buffer.as_bytes()[i + 1] == b'/' {
                        self.block_comment_depth -= 1;
                        if self.block_comment_depth == 0 {
                            self.state = LexState::Normal;
                        }
                        i += 2;
                    } else if ch == '/' && i + 1 < len && self.buffer.as_bytes()[i + 1] == b'*' {
                        self.block_comment_depth += 1;
                        i += 2;
                    } else {
                        i += char_len;
                    }
                }
            }
        }

        self.scan_offset = i;

        // Always compact the buffer when stmt_start has advanced past consumed
        // content (e.g., empty statements like `;;` that were skipped).
        if self.stmt_start > 0 {
            self.buffer = self.buffer[self.stmt_start..].to_string();
            self.scan_offset -= self.stmt_start;
            self.stmt_start = 0;
        }

        // If the remaining buffer is only whitespace and/or comments (no
        // meaningful CQL tokens), clear it so the REPL returns to the primary
        // prompt. This handles trailing line comments after semicolons
        // (e.g., `SELECT 1; -- comment`) and bare `;;`.
        if !self.buffer.is_empty() {
            let stripped = strip_comments(&self.buffer);
            if stripped.trim().is_empty() {
                self.buffer.clear();
                self.scan_offset = 0;
                self.state = LexState::Normal;
                self.block_comment_depth = 0;
            }
        }

        if statements.is_empty() {
            ParseResult::Incomplete
        } else {
            ParseResult::Complete(statements)
        }
    }
}

/// Decode the char at byte offset `i` in `s`, returning the char and its UTF-8 byte length.
fn decode_char_at(s: &str, i: usize) -> (char, usize) {
    // Safety: `i` must be at a char boundary, which our state machine guarantees
    // because we always advance by `char_len`.
    let ch = s[i..].chars().next().unwrap_or('\0');
    (ch, ch.len_utf8())
}

/// Return true if `text` is the opening of a CQL BATCH block.
///
/// Matches: `BEGIN BATCH`, `BEGIN UNLOGGED BATCH`, `BEGIN COUNTER BATCH`
/// (case-insensitive, any amount of internal whitespace).
fn starts_with_begin_batch(text: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    match words.as_slice() {
        [b, batch, ..]
            if b.eq_ignore_ascii_case("BEGIN") && batch.eq_ignore_ascii_case("BATCH") =>
        {
            true
        }
        [b, modifier, batch, ..]
            if b.eq_ignore_ascii_case("BEGIN")
                && (modifier.eq_ignore_ascii_case("UNLOGGED")
                    || modifier.eq_ignore_ascii_case("COUNTER"))
                && batch.eq_ignore_ascii_case("BATCH") =>
        {
            true
        }
        _ => false,
    }
}

/// Return true if `text` ends with the `APPLY BATCH` token pair.
fn ends_with_apply_batch(text: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    matches!(
        words.as_slice(),
        [.., apply, batch]
            if apply.eq_ignore_ascii_case("APPLY") && batch.eq_ignore_ascii_case("BATCH")
    )
}

/// Strip comments from a CQL fragment (used on extracted statements).
///
/// This function uses context-aware scanning to avoid stripping comment-like
/// sequences inside string literals (PR #150 fix). Handles nested block comments.
fn strip_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut state = LexState::Normal;
    let mut block_depth: usize = 0;
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let (ch, char_len) = decode_char_at(input, i);

        match state {
            LexState::Normal => {
                if ch == '\'' {
                    state = LexState::SingleQuote;
                    result.push(ch);
                    i += char_len;
                } else if ch == '"' {
                    state = LexState::DoubleQuote;
                    result.push(ch);
                    i += char_len;
                } else if ch == '$' && i + 1 < len && bytes[i + 1] == b'$' {
                    state = LexState::DollarQuote;
                    result.push_str("$$");
                    i += 2;
                } else if ch == '-' && i + 1 < len && bytes[i + 1] == b'-' {
                    // Line comment: skip to end of line
                    state = LexState::LineComment;
                    i += 2;
                } else if ch == '/' && i + 1 < len && bytes[i + 1] == b'*' {
                    // Block comment: skip content
                    state = LexState::BlockComment;
                    block_depth = 1;
                    i += 2;
                } else {
                    result.push(ch);
                    i += char_len;
                }
            }
            LexState::SingleQuote => {
                result.push(ch);
                if ch == '\'' {
                    if i + 1 < len && bytes[i + 1] == b'\'' {
                        result.push('\'');
                        i += 2;
                    } else {
                        state = LexState::Normal;
                        i += 1;
                    }
                } else {
                    i += char_len;
                }
            }
            LexState::DoubleQuote => {
                result.push(ch);
                if ch == '"' {
                    if i + 1 < len && bytes[i + 1] == b'"' {
                        result.push('"');
                        i += 2;
                    } else {
                        state = LexState::Normal;
                        i += 1;
                    }
                } else {
                    i += char_len;
                }
            }
            LexState::DollarQuote => {
                result.push(ch);
                if ch == '$' && i + 1 < len && bytes[i + 1] == b'$' {
                    result.push('$');
                    state = LexState::Normal;
                    i += 2;
                } else {
                    i += char_len;
                }
            }
            LexState::LineComment => {
                if ch == '\n' {
                    result.push('\n');
                    state = LexState::Normal;
                }
                i += char_len;
            }
            LexState::BlockComment => {
                if ch == '*' && i + 1 < len && bytes[i + 1] == b'/' {
                    block_depth -= 1;
                    if block_depth == 0 {
                        // Replace entire block comment with a space to avoid token merging
                        result.push(' ');
                        state = LexState::Normal;
                    }
                    i += 2;
                } else if ch == '/' && i + 1 < len && bytes[i + 1] == b'*' {
                    block_depth += 1;
                    i += 2;
                } else {
                    i += char_len;
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

    if is_shell_command(trimmed) {
        InputKind::ShellCommand(trimmed.to_string())
    } else {
        InputKind::CqlStatement(trimmed.to_string())
    }
}

/// Check if the first line of input looks like a shell command.
///
/// Used by the REPL to decide whether to wait for a semicolon
/// or dispatch immediately.
#[must_use]
pub fn is_shell_command(line: &str) -> bool {
    let trimmed = line.trim();
    // Strip trailing semicolon for command detection
    let without_semi = trimmed.strip_suffix(';').unwrap_or(trimmed).trim();
    let first_word = without_semi
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
#[must_use]
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
    let remaining = parser.remaining().trim();
    if !remaining.is_empty() {
        let stripped = strip_comments(remaining);
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

    #[test]
    fn empty_dollar_quote() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT $$$$;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT $$$$".to_string()])
        );
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

    #[test]
    fn line_comment_then_statement_across_lines() {
        let mut p = StatementParser::new();
        assert_eq!(p.feed_line("-- header comment"), ParseResult::Incomplete);
        let result = p.feed_line("SELECT 1;");
        assert_eq!(result, ParseResult::Complete(vec!["SELECT 1".to_string()]));
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

    #[test]
    fn block_comment_across_feed_lines() {
        let mut p = StatementParser::new();
        assert_eq!(p.feed_line("SELECT /* start"), ParseResult::Incomplete);
        assert_eq!(p.feed_line("still comment"), ParseResult::Incomplete);
        let result = p.feed_line("end */ 1;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT   1".to_string()])
        );
    }

    #[test]
    fn nested_block_comments() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT /* outer /* inner */ still comment */ 1;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT   1".to_string()])
        );
    }

    #[test]
    fn nested_block_comments_stripped() {
        let input = "SELECT /* outer /* inner */ still */ 1";
        let result = strip_comments(input);
        assert_eq!(result, "SELECT   1");
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
    fn shell_command_with_semicolon() {
        assert!(is_shell_command("USE my_ks;"));
        assert!(is_shell_command("HELP;"));
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
    fn classify_shell_command_with_semicolon() {
        assert_eq!(
            classify_input("USE my_ks;"),
            InputKind::ShellCommand("USE my_ks;".to_string())
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

    #[test]
    fn parse_batch_only_comments() {
        let input = "-- just a comment\n/* block */\n";
        let stmts = parse_batch(input);
        assert!(stmts.is_empty());
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

    // --- Parser reuse after Complete ---

    #[test]
    fn reuse_after_complete() {
        let mut p = StatementParser::new();
        let r1 = p.feed_line("SELECT 1;");
        assert_eq!(r1, ParseResult::Complete(vec!["SELECT 1".to_string()]));

        // Parser should work for subsequent statements
        let r2 = p.feed_line("SELECT 2;");
        assert_eq!(r2, ParseResult::Complete(vec!["SELECT 2".to_string()]));
    }

    #[test]
    fn reuse_after_complete_multiline() {
        let mut p = StatementParser::new();
        assert_eq!(
            p.feed_line("SELECT 1;"),
            ParseResult::Complete(vec!["SELECT 1".to_string()])
        );

        // Now a multi-line statement
        assert_eq!(p.feed_line("SELECT *"), ParseResult::Incomplete);
        let result = p.feed_line("FROM t;");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT *\nFROM t".to_string()])
        );
    }

    // --- Unterminated constructs ---

    #[test]
    fn unterminated_string_blocks_semicolon() {
        let stmts = parse_batch("SELECT 'unterminated;");
        assert!(stmts.is_empty());
    }

    #[test]
    fn unterminated_block_comment_blocks_semicolon() {
        let stmts = parse_batch("SELECT /* never closed;");
        assert!(stmts.is_empty());
    }

    // --- Backslash in strings ---

    #[test]
    fn backslash_in_string_is_literal() {
        // CQL does NOT use backslash escaping (uses '' instead)
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT '\\';");
        assert_eq!(
            result,
            ParseResult::Complete(vec!["SELECT '\\'".to_string()])
        );
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

    // --- Incremental scan correctness ---

    #[test]
    fn incremental_scan_preserves_state_across_lines() {
        // Verify that the parser doesn't re-scan from the start each time.
        // This is a correctness test: if state weren't preserved,
        // the second line's `'` would start a new string context.
        let mut p = StatementParser::new();
        assert_eq!(
            p.feed_line("INSERT INTO t VALUES ('multi"),
            ParseResult::Incomplete
        );
        assert_eq!(
            p.feed_line("line string with ; inside"),
            ParseResult::Incomplete
        );
        let result = p.feed_line("end of string');");
        assert_eq!(
            result,
            ParseResult::Complete(vec![
                "INSERT INTO t VALUES ('multi\nline string with ; inside\nend of string')"
                    .to_string()
            ])
        );
    }

    // --- BUG-7: Inline comment after semicolon ---

    #[test]
    fn inline_comment_after_semicolon_clears_buffer() {
        let mut p = StatementParser::new();
        let result = p.feed_line("SELECT 1; -- inline comment");
        assert_eq!(result, ParseResult::Complete(vec!["SELECT 1".to_string()]));
        // Parser should be empty — no continuation prompt
        assert!(p.is_empty());
    }

    #[test]
    fn inline_comment_after_semicolon_next_statement_works() {
        let mut p = StatementParser::new();
        let r1 = p.feed_line("SELECT 1; -- comment");
        assert_eq!(r1, ParseResult::Complete(vec!["SELECT 1".to_string()]));
        assert!(p.is_empty());

        // Next statement should work normally
        let r2 = p.feed_line("SELECT 2;");
        assert_eq!(r2, ParseResult::Complete(vec!["SELECT 2".to_string()]));
    }

    // --- BUG-8: Bare ;; enters continuation ---

    #[test]
    fn bare_semicolons_clear_buffer() {
        let mut p = StatementParser::new();
        let result = p.feed_line(";;");
        assert_eq!(result, ParseResult::Incomplete);
        // Parser should be empty — no continuation prompt
        assert!(p.is_empty());
    }

    #[test]
    fn bare_semicolons_then_statement() {
        let mut p = StatementParser::new();
        assert_eq!(p.feed_line(";;"), ParseResult::Incomplete);
        assert!(p.is_empty());

        let result = p.feed_line("SELECT 1;");
        assert_eq!(result, ParseResult::Complete(vec!["SELECT 1".to_string()]));
    }

    #[test]
    fn only_whitespace_and_comments_clears_buffer() {
        let mut p = StatementParser::new();
        assert_eq!(p.feed_line("-- just a comment"), ParseResult::Incomplete);
        assert!(p.is_empty());
    }

    #[test]
    fn block_comment_only_clears_buffer() {
        let mut p = StatementParser::new();
        assert_eq!(p.feed_line("/* block comment */"), ParseResult::Incomplete);
        assert!(p.is_empty());
    }
}

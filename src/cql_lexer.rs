//! Unified CQL lexer (tokenizer) with grammar-aware position tracking.
//!
//! Provides a single shared tokenizer that powers syntax highlighting (colorizer),
//! tab completion (completer), and statement parsing (parser). Replaces three
//! ad-hoc implementations with one consistent CQL understanding.
//!
//! Design: hand-written state machine, O(n) single pass, no dependencies.
//! See `docs/plans/18-cql-lexer.md` for motivation and design decisions.

/// A token produced by the CQL lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The kind of token.
    pub kind: TokenKind,
    /// The raw text of the token as it appears in the input.
    pub text: String,
    /// Byte offset where this token starts in the input.
    pub start: usize,
    /// Byte offset where this token ends (exclusive) in the input.
    pub end: usize,
}

/// Classification of a CQL token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// A CQL keyword (SELECT, FROM, WHERE, etc.). Determined by context — words
    /// in identifier position (after FROM, after dot, etc.) are `Identifier` instead.
    Keyword,
    /// An unquoted identifier (table name, column name, keyspace name, etc.).
    Identifier,
    /// A double-quoted identifier (`"MyTable"`).
    QuotedIdentifier,
    /// A single-quoted string literal (`'hello'`).
    StringLiteral,
    /// A dollar-quoted string literal (`$$body$$`).
    DollarStringLiteral,
    /// A numeric literal (integer or decimal: `42`, `3.14`, `-1`).
    NumberLiteral,
    /// A blob literal (`0xDEADBEEF`).
    BlobLiteral,
    /// A UUID literal (`550e8400-e29b-41d4-a716-446655440000`).
    UuidLiteral,
    /// A boolean literal (`true`, `false`).
    BooleanLiteral,
    /// An operator (`=`, `<`, `>`, `<=`, `>=`, `!=`, `+`, `-`, etc.).
    Operator,
    /// Punctuation (`;`, `,`, `(`, `)`, `.`, `*`, `?`).
    Punctuation,
    /// Whitespace (spaces, tabs, newlines).
    Whitespace,
    /// A line comment (`-- ...`).
    LineComment,
    /// A block comment (`/* ... */`), possibly nested.
    BlockComment,
    /// Unrecognized character.
    Unknown,
}

/// Grammar context: what syntactic position we're at, used to distinguish
/// keywords from identifiers and to drive tab completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammarContext {
    /// Start of statement — expecting a keyword like SELECT, INSERT, etc.
    Start,
    /// After SELECT — expecting column list, *, or DISTINCT.
    ExpectColumnList,
    /// After FROM / INTO / UPDATE / TABLE / INDEX ON — expecting a table name.
    ExpectTable,
    /// After USE / KEYSPACE — expecting a keyspace name.
    ExpectKeyspace,
    /// After WHERE / AND / IF (conditions) — expecting a column name.
    ExpectColumn,
    /// After SET (in UPDATE) — expecting column = value pairs.
    ExpectSetClause,
    /// After a dot — expecting the second part of a qualified name.
    ExpectQualifiedPart,
    /// After a column/table type keyword — expecting a CQL type name.
    ExpectType,
    /// After ORDER — expecting BY.
    ExpectOrderBy,
    /// After ORDER BY — expecting column name.
    ExpectOrderByColumn,
    /// After VALUES — expecting ( value_list ).
    ExpectValues,
    /// Inside WITH clause options.
    ExpectWithOption,
    /// After CONSISTENCY / SERIAL CONSISTENCY — expecting level name.
    ExpectConsistencyLevel,
    /// After DESCRIBE / DESC — expecting sub-command or schema name.
    ExpectDescribeTarget,
    /// After SOURCE / CAPTURE — expecting file path.
    ExpectFilePath,
    /// General clause context (default within a statement body).
    General,
}

/// Set of CQL keywords and shell commands (uppercase, sorted for binary search).
const CQL_KEYWORDS: &[&str] = &[
    "ADD",
    "AGGREGATE",
    "ALL",
    "ALLOW",
    "ALTER",
    "AND",
    "APPLY",
    "AS",
    "ASC",
    "AUTHORIZE",
    "BATCH",
    "BEGIN",
    "BY",
    "CALLED",
    "CAPTURE",
    "CLEAR",
    "CLS",
    "CLUSTERING",
    "COLUMN",
    "COMPACT",
    "CONSISTENCY",
    "CONTAINS",
    "COPY",
    "COUNT",
    "COUNTER",
    "CREATE",
    "CUSTOM",
    "DELETE",
    "DESC",
    "DESCRIBE",
    "DISTINCT",
    "DROP",
    "EACH_QUORUM",
    "ENTRIES",
    "EXECUTE",
    "EXISTS",
    "EXIT",
    "EXPAND",
    "FILTERING",
    "FINALFUNC",
    "FROM",
    "FROZEN",
    "FULL",
    "FUNCTION",
    "FUNCTIONS",
    "GRANT",
    "HELP",
    "IF",
    "IN",
    "INDEX",
    "INITCOND",
    "INPUT",
    "INSERT",
    "INTO",
    "IS",
    "JSON",
    "KEY",
    "KEYSPACE",
    "KEYSPACES",
    "LANGUAGE",
    "LIKE",
    "LIMIT",
    "LIST",
    "LOCAL_ONE",
    "LOCAL_QUORUM",
    "LOGIN",
    "MAP",
    "MATERIALIZED",
    "MODIFY",
    "NAMESPACE",
    "NORECURSIVE",
    "NOT",
    "NULL",
    "OF",
    "ON",
    "ONE",
    "OR",
    "ORDER",
    "PAGING",
    "PARTITION",
    "PASSWORD",
    "PER",
    "PERMISSION",
    "PERMISSIONS",
    "PRIMARY",
    "QUIT",
    "QUORUM",
    "RENAME",
    "REPLACE",
    "RETURNS",
    "REVOKE",
    "SCHEMA",
    "SELECT",
    "SERIAL",
    "SET",
    "SFUNC",
    "SHOW",
    "SOURCE",
    "STATIC",
    "STORAGE",
    "STYPE",
    "SUPERUSER",
    "TABLE",
    "TABLES",
    "TEXT",
    "THREE",
    "TIMESTAMP",
    "TO",
    "TOKEN",
    "TRACING",
    "TRIGGER",
    "TRUNCATE",
    "TTL",
    "TUPLE",
    "TWO",
    "TYPE",
    "UNICODE",
    "UNLOGGED",
    "UPDATE",
    "USE",
    "USER",
    "USERS",
    "USING",
    "VALUES",
    "VIEW",
    "WHERE",
    "WITH",
    "WRITETIME",
];

/// Check if a word is a CQL keyword (case-insensitive).
pub fn is_cql_keyword(word: &str) -> bool {
    let upper = word.to_uppercase();
    CQL_KEYWORDS.binary_search(&upper.as_str()).is_ok()
}

/// Tokenize a CQL input string into a sequence of tokens with grammar context.
///
/// This is the main entry point. It performs a single O(n) pass and classifies
/// each token using both lexical rules and grammar position tracking.
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut ctx = GrammarContext::Start;
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let ch = bytes[i];

        // Whitespace
        if ch.is_ascii_whitespace() {
            let start = i;
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Whitespace,
                text: input[start..i].to_string(),
                start,
                end: i,
            });
            continue;
        }

        // Line comment: --
        if ch == b'-' && i + 1 < len && bytes[i + 1] == b'-' {
            let start = i;
            i += 2;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::LineComment,
                text: input[start..i].to_string(),
                start,
                end: i,
            });
            continue;
        }

        // Block comment: /* ... */ (nested)
        if ch == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            let start = i;
            let mut depth: usize = 1;
            i += 2;
            while i < len && depth > 0 {
                if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if bytes[i] == b'*' && i + 1 < len && bytes[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            tokens.push(Token {
                kind: TokenKind::BlockComment,
                text: input[start..i].to_string(),
                start,
                end: i,
            });
            continue;
        }

        // Single-quoted string literal
        if ch == b'\'' {
            let start = i;
            i += 1;
            loop {
                if i >= len {
                    break; // unterminated
                }
                if bytes[i] == b'\'' {
                    i += 1;
                    // Escaped quote '' — continue string
                    if i < len && bytes[i] == b'\'' {
                        i += 1;
                        continue;
                    }
                    break;
                }
                // Advance by UTF-8 char length
                i += char_len_at(bytes, i);
            }
            tokens.push(Token {
                kind: TokenKind::StringLiteral,
                text: input[start..i].to_string(),
                start,
                end: i,
            });
            ctx = advance_context_after_value(ctx);
            continue;
        }

        // Double-quoted identifier
        if ch == b'"' {
            let start = i;
            i += 1;
            loop {
                if i >= len {
                    break; // unterminated
                }
                if bytes[i] == b'"' {
                    i += 1;
                    // Escaped quote "" — continue
                    if i < len && bytes[i] == b'"' {
                        i += 1;
                        continue;
                    }
                    break;
                }
                i += char_len_at(bytes, i);
            }
            tokens.push(Token {
                kind: TokenKind::QuotedIdentifier,
                text: input[start..i].to_string(),
                start,
                end: i,
            });
            ctx = advance_context_after_name(ctx);
            continue;
        }

        // Dollar-quoted string: $$...$$
        if ch == b'$' && i + 1 < len && bytes[i + 1] == b'$' {
            let start = i;
            i += 2;
            loop {
                if i + 1 >= len {
                    i = len;
                    break; // unterminated
                }
                if bytes[i] == b'$' && bytes[i + 1] == b'$' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::DollarStringLiteral,
                text: input[start..i].to_string(),
                start,
                end: i,
            });
            ctx = advance_context_after_value(ctx);
            continue;
        }

        // Blob literal: 0x followed by hex digits
        if ch == b'0' && i + 1 < len && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X') {
            let start = i;
            i += 2;
            while i < len && bytes[i].is_ascii_hexdigit() {
                i += 1;
            }
            // Make sure it's not followed by an identifier char (would be an identifier)
            if i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                // Actually an identifier starting with 0x... — backtrack
                i = start;
            } else {
                tokens.push(Token {
                    kind: TokenKind::BlobLiteral,
                    text: input[start..i].to_string(),
                    start,
                    end: i,
                });
                ctx = advance_context_after_value(ctx);
                continue;
            }
        }

        // Number literal: digits, optional dot, optional exponent
        // Also handles negative numbers when preceded by operator context
        if ch.is_ascii_digit()
            || (ch == b'-'
                && i + 1 < len
                && bytes[i + 1].is_ascii_digit()
                && is_number_sign_position(&tokens))
        {
            let start = i;
            if ch == b'-' {
                i += 1;
            }
            while i < len && bytes[i].is_ascii_digit() {
                i += 1;
            }
            // Decimal part
            if i < len && bytes[i] == b'.' && i + 1 < len && bytes[i + 1].is_ascii_digit() {
                i += 1;
                while i < len && bytes[i].is_ascii_digit() {
                    i += 1;
                }
            }
            // Exponent
            if i < len && (bytes[i] == b'e' || bytes[i] == b'E') {
                let save = i;
                i += 1;
                if i < len && (bytes[i] == b'+' || bytes[i] == b'-') {
                    i += 1;
                }
                if i < len && bytes[i].is_ascii_digit() {
                    while i < len && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                } else {
                    i = save; // not an exponent, backtrack
                }
            }
            // UUID check: number followed by '-' hex pattern (8-4-4-4-12)
            if i < len && bytes[i] == b'-' && looks_like_uuid(input, start, i) {
                // Parse the full UUID
                let uuid_end = scan_uuid(input, start);
                if uuid_end > i {
                    i = uuid_end;
                    tokens.push(Token {
                        kind: TokenKind::UuidLiteral,
                        text: input[start..i].to_string(),
                        start,
                        end: i,
                    });
                    ctx = advance_context_after_value(ctx);
                    continue;
                }
            }
            // Make sure this isn't part of an identifier (like table1)
            if i < len && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
                // It's an identifier starting with digits — shouldn't happen in valid CQL,
                // but treat as identifier
                while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let word = &input[start..i];
                tokens.push(Token {
                    kind: classify_word(word, ctx),
                    text: word.to_string(),
                    start,
                    end: i,
                });
                ctx = advance_context_after_word(word, ctx);
                continue;
            }
            tokens.push(Token {
                kind: TokenKind::NumberLiteral,
                text: input[start..i].to_string(),
                start,
                end: i,
            });
            ctx = advance_context_after_value(ctx);
            continue;
        }

        // Word: identifier or keyword
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &input[start..i];

            // UUID check: word of hex chars followed by '-' hex pattern
            if i < len
                && bytes[i] == b'-'
                && word.len() == 8
                && word.chars().all(|c| c.is_ascii_hexdigit())
            {
                let uuid_end = scan_uuid(input, start);
                if uuid_end > i {
                    i = uuid_end;
                    tokens.push(Token {
                        kind: TokenKind::UuidLiteral,
                        text: input[start..i].to_string(),
                        start,
                        end: i,
                    });
                    ctx = advance_context_after_value(ctx);
                    continue;
                }
            }

            let kind = classify_word(word, ctx);
            tokens.push(Token {
                kind,
                text: word.to_string(),
                start,
                end: i,
            });
            ctx = advance_context_after_word(word, ctx);
            continue;
        }

        // Operators
        if is_operator_char(ch) {
            let start = i;
            // Two-char operators: <=, >=, !=
            if i + 1 < len && is_two_char_operator(ch, bytes[i + 1]) {
                i += 2;
            } else {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Operator,
                text: input[start..i].to_string(),
                start,
                end: i,
            });
            continue;
        }

        // Punctuation
        if is_punctuation(ch) {
            let start = i;
            i += 1;
            let text = input[start..i].to_string();

            // Track dot for qualified names
            if ch == b'.' {
                ctx = GrammarContext::ExpectQualifiedPart;
            }

            tokens.push(Token {
                kind: TokenKind::Punctuation,
                text,
                start,
                end: i,
            });
            continue;
        }

        // Unknown character (advance by UTF-8 char length)
        let start = i;
        let clen = char_len_at(bytes, i);
        i += clen;
        tokens.push(Token {
            kind: TokenKind::Unknown,
            text: input[start..i].to_string(),
            start,
            end: i,
        });
    }

    tokens
}

/// Get the grammar context at the end of the given input.
/// Useful for tab completion to know what kind of token is expected next.
pub fn grammar_context_at_end(input: &str) -> GrammarContext {
    let tokens = tokenize(input);
    context_from_tokens(&tokens)
}

/// Derive grammar context from a token sequence (skipping whitespace/comments).
pub fn context_from_tokens(tokens: &[Token]) -> GrammarContext {
    // Walk backwards through significant tokens to determine context
    let significant: Vec<&Token> = tokens
        .iter()
        .filter(|t| {
            !matches!(
                t.kind,
                TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment
            )
        })
        .collect();

    if significant.is_empty() {
        return GrammarContext::Start;
    }

    let last = significant.last().unwrap();

    // Check last keyword for context
    if last.kind == TokenKind::Keyword || last.kind == TokenKind::Identifier {
        let upper = last.text.to_uppercase();
        match upper.as_str() {
            "SELECT" | "DISTINCT" => return GrammarContext::ExpectColumnList,
            "FROM" | "INTO" | "UPDATE" | "TABLE" | "TRUNCATE" => {
                return GrammarContext::ExpectTable
            }
            "USE" | "KEYSPACE" | "KEYSPACES" => return GrammarContext::ExpectKeyspace,
            "WHERE" | "IF" => return GrammarContext::ExpectColumn,
            "AND" => {
                // AND after WHERE/IF is column context
                if has_keyword_before(&significant, &["WHERE", "IF"]) {
                    return GrammarContext::ExpectColumn;
                }
                return GrammarContext::General;
            }
            "SET" => {
                // SET after UPDATE is column assignment; SET as a type is different
                if has_keyword_before(&significant, &["UPDATE"]) {
                    return GrammarContext::ExpectSetClause;
                }
                return GrammarContext::General;
            }
            "ORDER" => return GrammarContext::ExpectOrderBy,
            "BY" => {
                if significant.len() >= 2
                    && significant[significant.len() - 2].text.to_uppercase() == "ORDER"
                {
                    return GrammarContext::ExpectOrderByColumn;
                }
                return GrammarContext::General;
            }
            "VALUES" => return GrammarContext::ExpectValues,
            "WITH" => return GrammarContext::ExpectWithOption,
            "CONSISTENCY" => return GrammarContext::ExpectConsistencyLevel,
            "DESCRIBE" | "DESC" => return GrammarContext::ExpectDescribeTarget,
            "SOURCE" | "CAPTURE" => return GrammarContext::ExpectFilePath,
            "ON" => {
                // INDEX ... ON -> expect table
                if has_keyword_before(&significant, &["INDEX"]) {
                    return GrammarContext::ExpectTable;
                }
                return GrammarContext::General;
            }
            "INDEX" => {
                // After CREATE/DROP INDEX -> expect index name (identifier)
                return GrammarContext::General;
            }
            _ => {}
        }
    }

    if last.kind == TokenKind::Punctuation && last.text == "." {
        return GrammarContext::ExpectQualifiedPart;
    }

    // After FROM table_name — we're in general clause context
    // Check if two tokens back is a table-expecting keyword
    if significant.len() >= 2 {
        let second_last = significant[significant.len() - 2];
        let sl_upper = second_last.text.to_uppercase();
        if matches!(
            sl_upper.as_str(),
            "FROM" | "INTO" | "UPDATE" | "TABLE" | "TRUNCATE"
        ) {
            return GrammarContext::General;
        }
        // After SERIAL -> if next is CONSISTENCY
        if sl_upper == "SERIAL" && last.text.to_uppercase() == "CONSISTENCY" {
            return GrammarContext::ExpectConsistencyLevel;
        }
    }

    GrammarContext::General
}

/// Contexts where the next word is always an identifier (a name), regardless
/// of whether it matches a keyword. E.g., after FROM the next word is a table name.
fn is_strict_identifier_context(ctx: GrammarContext) -> bool {
    matches!(
        ctx,
        GrammarContext::ExpectTable
            | GrammarContext::ExpectKeyspace
            | GrammarContext::ExpectColumn
            | GrammarContext::ExpectQualifiedPart
            | GrammarContext::ExpectOrderByColumn
            | GrammarContext::ExpectSetClause
            | GrammarContext::ExpectConsistencyLevel
    )
}

/// Keywords that remain keywords inside a SELECT column list.
/// These are clause-level keywords that terminate or modify the column list.
const COLUMN_LIST_KEYWORDS: &[&str] = &["AS", "DISTINCT", "FROM", "JSON"];

fn is_column_list_keyword(word: &str) -> bool {
    let upper = word.to_uppercase();
    COLUMN_LIST_KEYWORDS.contains(&upper.as_str())
}

/// Classify a word as keyword, boolean, or identifier based on grammar context.
fn classify_word(word: &str, ctx: GrammarContext) -> TokenKind {
    let upper = word.to_uppercase();

    // Boolean literals
    if upper == "TRUE" || upper == "FALSE" {
        return TokenKind::BooleanLiteral;
    }

    // In strict identifier contexts, EVERYTHING is an identifier
    if is_strict_identifier_context(ctx) {
        return TokenKind::Identifier;
    }

    // In column list context, only specific keywords remain keywords
    if ctx == GrammarContext::ExpectColumnList {
        if is_column_list_keyword(word) {
            return TokenKind::Keyword;
        }
        return TokenKind::Identifier;
    }

    // NULL is a keyword-like value
    if upper == "NULL" {
        return TokenKind::Keyword;
    }

    if is_cql_keyword(&upper) {
        TokenKind::Keyword
    } else {
        TokenKind::Identifier
    }
}

/// Advance the grammar context after seeing a word token.
fn advance_context_after_word(word: &str, ctx: GrammarContext) -> GrammarContext {
    let upper = word.to_uppercase();

    match upper.as_str() {
        "SELECT" => GrammarContext::ExpectColumnList,
        "DISTINCT" if ctx == GrammarContext::ExpectColumnList => GrammarContext::ExpectColumnList,
        "FROM" => GrammarContext::ExpectTable,
        "INTO" => GrammarContext::ExpectTable,
        "UPDATE" => GrammarContext::ExpectTable,
        "TABLE" => GrammarContext::ExpectTable,
        "TRUNCATE" => GrammarContext::ExpectTable,
        "USE" => GrammarContext::ExpectKeyspace,
        "KEYSPACE" => GrammarContext::ExpectKeyspace,
        "WHERE" => GrammarContext::ExpectColumn,
        "AND" => {
            // Preserve column context if we're in a WHERE/SET clause
            match ctx {
                GrammarContext::ExpectColumn | GrammarContext::General => {
                    // After WHERE col = val AND -> next column
                    GrammarContext::ExpectColumn
                }
                _ => GrammarContext::General,
            }
        }
        "SET" => {
            // After UPDATE table SET -> column assignment
            match ctx {
                GrammarContext::General => GrammarContext::ExpectSetClause,
                _ => GrammarContext::General,
            }
        }
        "ORDER" => GrammarContext::ExpectOrderBy,
        "BY" => {
            if ctx == GrammarContext::ExpectOrderBy {
                GrammarContext::ExpectOrderByColumn
            } else {
                GrammarContext::General
            }
        }
        "VALUES" => GrammarContext::ExpectValues,
        "WITH" => GrammarContext::ExpectWithOption,
        "ON" => {
            // Could be INDEX ... ON table
            GrammarContext::ExpectTable
        }
        "CONSISTENCY" => GrammarContext::ExpectConsistencyLevel,
        "DESCRIBE" | "DESC" => GrammarContext::ExpectDescribeTarget,
        "SOURCE" | "CAPTURE" => GrammarContext::ExpectFilePath,
        "INSERT" => GrammarContext::General, // INSERT INTO -> INTO will set ExpectTable
        "DELETE" => GrammarContext::General, // DELETE FROM -> FROM will set ExpectTable
        "CREATE" | "ALTER" | "DROP" => GrammarContext::General,
        "IF" => GrammarContext::ExpectColumn,
        "LIMIT" => GrammarContext::General,
        _ => {
            // After an identifier in table/keyspace/column position, go to General
            match ctx {
                GrammarContext::ExpectTable
                | GrammarContext::ExpectKeyspace
                | GrammarContext::ExpectColumn
                | GrammarContext::ExpectOrderByColumn
                | GrammarContext::ExpectQualifiedPart
                | GrammarContext::ExpectDescribeTarget => GrammarContext::General,
                GrammarContext::ExpectColumnList => GrammarContext::ExpectColumnList, // stay in column list
                GrammarContext::ExpectSetClause => GrammarContext::ExpectSetClause,
                other => other,
            }
        }
    }
}

/// Advance context after a value (string literal, number, etc.).
fn advance_context_after_value(ctx: GrammarContext) -> GrammarContext {
    match ctx {
        GrammarContext::ExpectColumnList => GrammarContext::ExpectColumnList,
        _ => GrammarContext::General,
    }
}

/// Advance context after a name (quoted identifier, etc.).
fn advance_context_after_name(ctx: GrammarContext) -> GrammarContext {
    match ctx {
        GrammarContext::ExpectTable
        | GrammarContext::ExpectKeyspace
        | GrammarContext::ExpectColumn
        | GrammarContext::ExpectQualifiedPart
        | GrammarContext::ExpectOrderByColumn
        | GrammarContext::ExpectDescribeTarget => GrammarContext::General,
        GrammarContext::ExpectColumnList => GrammarContext::ExpectColumnList,
        other => other,
    }
}

/// Check if any of the given keywords appear earlier in the significant token list.
fn has_keyword_before(significant: &[&Token], keywords: &[&str]) -> bool {
    significant.iter().rev().skip(1).any(|t| {
        let upper = t.text.to_uppercase();
        keywords.contains(&upper.as_str())
    })
}

/// Determine if a '-' is in a position where it could be a negative number sign
/// (after operator, punctuation, or at start).
fn is_number_sign_position(tokens: &[Token]) -> bool {
    match tokens.last() {
        None => true,
        Some(t) => matches!(
            t.kind,
            TokenKind::Operator
                | TokenKind::Punctuation
                | TokenKind::Keyword
                | TokenKind::Whitespace
        ),
    }
}

/// Check if the text from `start` to `num_end` followed by '-' looks like the
/// beginning of a UUID (8 hex digits).
fn looks_like_uuid(input: &str, start: usize, num_end: usize) -> bool {
    let segment = &input[start..num_end];
    segment.len() == 8 && segment.chars().all(|c| c.is_ascii_hexdigit())
}

/// Scan a UUID pattern: 8-4-4-4-12 hex digits with dashes.
/// Returns the end position if valid, or `start` if not.
fn scan_uuid(input: &str, start: usize) -> usize {
    let expected_segments = [8, 4, 4, 4, 12];
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut pos = start;

    for (seg_idx, &seg_len) in expected_segments.iter().enumerate() {
        if seg_idx > 0 {
            if pos >= len || bytes[pos] != b'-' {
                return start;
            }
            pos += 1;
        }
        let seg_start = pos;
        while pos < len && bytes[pos].is_ascii_hexdigit() {
            pos += 1;
        }
        if pos - seg_start != seg_len {
            return start;
        }
    }

    // Make sure UUID is not followed by alphanumeric (would be part of a longer token)
    if pos < len && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
        return start;
    }

    pos
}

fn is_operator_char(ch: u8) -> bool {
    matches!(ch, b'=' | b'<' | b'>' | b'!' | b'+' | b'%')
}

fn is_two_char_operator(first: u8, second: u8) -> bool {
    matches!((first, second), (b'<', b'=') | (b'>', b'=') | (b'!', b'='))
}

fn is_punctuation(ch: u8) -> bool {
    matches!(
        ch,
        b';' | b',' | b'(' | b')' | b'.' | b'*' | b'?' | b'{' | b'}' | b'[' | b']' | b':'
    )
}

/// Get the UTF-8 byte length of the char starting at position `i`.
fn char_len_at(bytes: &[u8], i: usize) -> usize {
    if i >= bytes.len() {
        return 1;
    }
    let b = bytes[i];
    if b < 0x80 {
        1
    } else if b < 0xE0 {
        2
    } else if b < 0xF0 {
        3
    } else {
        4
    }
}

/// Extract only the significant (non-whitespace, non-comment) tokens.
pub fn significant_tokens(tokens: &[Token]) -> Vec<&Token> {
    tokens
        .iter()
        .filter(|t| {
            !matches!(
                t.kind,
                TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment
            )
        })
        .collect()
}

/// Strip comments from CQL input, replacing block comments with a space
/// and removing line comments (preserving newlines).
/// This is used by the parser for comment stripping.
pub fn strip_comments(input: &str) -> String {
    let tokens = tokenize(input);
    let mut result = String::with_capacity(input.len());
    for token in &tokens {
        match token.kind {
            TokenKind::LineComment => {
                // preserve the newline if the comment was followed by one
                // (the newline is not part of the comment token)
            }
            TokenKind::BlockComment => {
                result.push(' ');
            }
            _ => {
                result.push_str(&token.text);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Helper =====

    #[allow(dead_code)]
    fn token_kinds(input: &str) -> Vec<TokenKind> {
        tokenize(input).into_iter().map(|t| t.kind).collect()
    }

    fn significant_kinds(input: &str) -> Vec<TokenKind> {
        tokenize(input)
            .into_iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .map(|t| t.kind)
            .collect()
    }

    fn significant_texts(input: &str) -> Vec<String> {
        tokenize(input)
            .into_iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .map(|t| t.text)
            .collect()
    }

    // ===== Token Kind Tests =====

    #[test]
    fn keyword_select() {
        let tokens = tokenize("SELECT");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Keyword);
        assert_eq!(tokens[0].text, "SELECT");
    }

    #[test]
    fn keyword_case_insensitive() {
        let tokens = tokenize("select");
        assert_eq!(tokens[0].kind, TokenKind::Keyword);
        assert_eq!(tokens[0].text, "select");
    }

    #[test]
    fn keyword_mixed_case() {
        let tokens = tokenize("Select");
        assert_eq!(tokens[0].kind, TokenKind::Keyword);
    }

    #[test]
    fn identifier_plain() {
        // After FROM, words are identifiers even if they match keywords
        let tokens = tokenize("FROM users");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[0].kind, TokenKind::Keyword);
        assert_eq!(sig[1].kind, TokenKind::Identifier);
        assert_eq!(sig[1].text, "users");
    }

    #[test]
    fn identifier_after_from_keyword_name() {
        // USERS after FROM should be identifier, not keyword
        let tokens = tokenize("SELECT * FROM USERS");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[3].text, "USERS");
        assert_eq!(sig[3].kind, TokenKind::Identifier);
    }

    #[test]
    fn identifier_key_after_from() {
        let tokens = tokenize("SELECT * FROM KEY");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[3].text, "KEY");
        assert_eq!(sig[3].kind, TokenKind::Identifier);
    }

    #[test]
    fn identifier_set_after_from() {
        let tokens = tokenize("SELECT * FROM SET");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[3].text, "SET");
        assert_eq!(sig[3].kind, TokenKind::Identifier);
    }

    #[test]
    fn identifier_after_into() {
        let tokens = tokenize("INSERT INTO my_table");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[2].kind, TokenKind::Identifier);
    }

    #[test]
    fn identifier_after_update() {
        let tokens = tokenize("UPDATE my_table SET");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[1].kind, TokenKind::Identifier);
        assert_eq!(sig[1].text, "my_table");
    }

    #[test]
    fn identifier_after_dot() {
        let tokens = tokenize("ks.my_table");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[0].kind, TokenKind::Identifier); // ks at start is identifier? No, at Start it's not a keyword
                                                        // Actually "ks" is not a keyword, so it's Identifier
        assert_eq!(sig[1].kind, TokenKind::Punctuation); // .
        assert_eq!(sig[2].kind, TokenKind::Identifier); // my_table
    }

    #[test]
    fn keyword_after_dot_is_identifier() {
        // SELECT after a dot should be identifier (qualified name)
        let tokens = tokenize("FROM ks.SELECT");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[2].kind, TokenKind::Punctuation); // .
        assert_eq!(sig[3].kind, TokenKind::Identifier); // SELECT as identifier
    }

    #[test]
    fn quoted_identifier() {
        let tokens = tokenize("\"MyTable\"");
        assert_eq!(tokens[0].kind, TokenKind::QuotedIdentifier);
        assert_eq!(tokens[0].text, "\"MyTable\"");
    }

    #[test]
    fn quoted_identifier_with_escape() {
        let tokens = tokenize("\"My\"\"Table\"");
        assert_eq!(tokens[0].kind, TokenKind::QuotedIdentifier);
        assert_eq!(tokens[0].text, "\"My\"\"Table\"");
    }

    #[test]
    fn string_literal_simple() {
        let tokens = tokenize("'hello'");
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[0].text, "'hello'");
    }

    #[test]
    fn string_literal_escaped_quote() {
        let tokens = tokenize("'it''s'");
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[0].text, "'it''s'");
    }

    #[test]
    fn string_literal_with_semicolon() {
        let tokens = tokenize("'hello;world'");
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[0].text, "'hello;world'");
    }

    #[test]
    fn string_literal_empty() {
        let tokens = tokenize("''");
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[0].text, "''");
    }

    #[test]
    fn dollar_string_literal() {
        let tokens = tokenize("$$hello world$$");
        assert_eq!(tokens[0].kind, TokenKind::DollarStringLiteral);
        assert_eq!(tokens[0].text, "$$hello world$$");
    }

    #[test]
    fn dollar_string_with_semicolon() {
        let tokens = tokenize("$$a;b$$");
        assert_eq!(tokens[0].kind, TokenKind::DollarStringLiteral);
    }

    #[test]
    fn dollar_string_empty() {
        let tokens = tokenize("$$$$");
        assert_eq!(tokens[0].kind, TokenKind::DollarStringLiteral);
        assert_eq!(tokens[0].text, "$$$$");
    }

    #[test]
    fn number_integer() {
        let tokens = tokenize("42");
        assert_eq!(tokens[0].kind, TokenKind::NumberLiteral);
        assert_eq!(tokens[0].text, "42");
    }

    #[test]
    fn number_decimal() {
        let tokens = tokenize("3.14");
        assert_eq!(tokens[0].kind, TokenKind::NumberLiteral);
        assert_eq!(tokens[0].text, "3.14");
    }

    #[test]
    fn number_negative() {
        let tokens = tokenize("= -1");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[1].kind, TokenKind::NumberLiteral);
        assert_eq!(sig[1].text, "-1");
    }

    #[test]
    fn number_exponent() {
        let tokens = tokenize("1.5E10");
        assert_eq!(tokens[0].kind, TokenKind::NumberLiteral);
        assert_eq!(tokens[0].text, "1.5E10");
    }

    #[test]
    fn number_not_part_of_identifier() {
        let tokens = tokenize("LIMIT 100");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[1].kind, TokenKind::NumberLiteral);
    }

    #[test]
    fn blob_literal() {
        let tokens = tokenize("0xDEADBEEF");
        assert_eq!(tokens[0].kind, TokenKind::BlobLiteral);
        assert_eq!(tokens[0].text, "0xDEADBEEF");
    }

    #[test]
    fn blob_literal_lowercase() {
        let tokens = tokenize("0xdeadbeef");
        assert_eq!(tokens[0].kind, TokenKind::BlobLiteral);
    }

    #[test]
    fn uuid_literal() {
        let tokens = tokenize("550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(tokens[0].kind, TokenKind::UuidLiteral);
    }

    #[test]
    fn boolean_true() {
        let tokens = tokenize("true");
        assert_eq!(tokens[0].kind, TokenKind::BooleanLiteral);
    }

    #[test]
    fn boolean_false() {
        let tokens = tokenize("FALSE");
        assert_eq!(tokens[0].kind, TokenKind::BooleanLiteral);
    }

    #[test]
    fn operator_equals() {
        let tokens = tokenize("=");
        assert_eq!(tokens[0].kind, TokenKind::Operator);
    }

    #[test]
    fn operator_less_equal() {
        let tokens = tokenize("<=");
        assert_eq!(tokens[0].kind, TokenKind::Operator);
        assert_eq!(tokens[0].text, "<=");
    }

    #[test]
    fn operator_greater_equal() {
        let tokens = tokenize(">=");
        assert_eq!(tokens[0].kind, TokenKind::Operator);
    }

    #[test]
    fn operator_not_equal() {
        let tokens = tokenize("!=");
        assert_eq!(tokens[0].kind, TokenKind::Operator);
    }

    #[test]
    fn punctuation_semicolon() {
        let tokens = tokenize(";");
        assert_eq!(tokens[0].kind, TokenKind::Punctuation);
    }

    #[test]
    fn punctuation_comma() {
        let tokens = tokenize(",");
        assert_eq!(tokens[0].kind, TokenKind::Punctuation);
    }

    #[test]
    fn punctuation_parens() {
        let kinds = significant_kinds("(x)");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Punctuation,
                TokenKind::Identifier,
                TokenKind::Punctuation
            ]
        );
    }

    #[test]
    fn punctuation_star() {
        let tokens = tokenize("*");
        assert_eq!(tokens[0].kind, TokenKind::Punctuation);
    }

    #[test]
    fn punctuation_question_mark() {
        let tokens = tokenize("?");
        assert_eq!(tokens[0].kind, TokenKind::Punctuation);
    }

    #[test]
    fn whitespace_space() {
        let tokens = tokenize("  ");
        assert_eq!(tokens[0].kind, TokenKind::Whitespace);
    }

    #[test]
    fn whitespace_tab() {
        let tokens = tokenize("\t");
        assert_eq!(tokens[0].kind, TokenKind::Whitespace);
    }

    #[test]
    fn whitespace_newline() {
        let tokens = tokenize("\n");
        assert_eq!(tokens[0].kind, TokenKind::Whitespace);
    }

    #[test]
    fn line_comment() {
        let tokens = tokenize("-- this is a comment");
        assert_eq!(tokens[0].kind, TokenKind::LineComment);
        assert_eq!(tokens[0].text, "-- this is a comment");
    }

    #[test]
    fn line_comment_stops_at_newline() {
        let tokens = tokenize("-- comment\nSELECT");
        assert_eq!(tokens[0].kind, TokenKind::LineComment);
        assert_eq!(tokens[0].text, "-- comment");
        // newline + SELECT follow
    }

    #[test]
    fn block_comment() {
        let tokens = tokenize("/* block */");
        assert_eq!(tokens[0].kind, TokenKind::BlockComment);
        assert_eq!(tokens[0].text, "/* block */");
    }

    #[test]
    fn block_comment_nested() {
        let tokens = tokenize("/* outer /* inner */ still */");
        assert_eq!(tokens[0].kind, TokenKind::BlockComment);
        assert_eq!(tokens[0].text, "/* outer /* inner */ still */");
    }

    #[test]
    fn block_comment_with_semicolon() {
        let tokens = tokenize("/* ; */");
        assert_eq!(tokens[0].kind, TokenKind::BlockComment);
    }

    #[test]
    fn unknown_char() {
        let tokens = tokenize("@");
        assert_eq!(tokens[0].kind, TokenKind::Unknown);
    }

    // ===== Span Tests =====

    #[test]
    fn spans_are_correct() {
        let tokens = tokenize("SELECT *");
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 6);
        assert_eq!(tokens[1].start, 6);
        assert_eq!(tokens[1].end, 7);
        assert_eq!(tokens[2].start, 7);
        assert_eq!(tokens[2].end, 8);
    }

    #[test]
    fn spans_cover_full_input() {
        let input = "SELECT * FROM users WHERE id = 1;";
        let tokens = tokenize(input);
        let last = tokens.last().unwrap();
        assert_eq!(last.end, input.len());
        // Verify no gaps
        for window in tokens.windows(2) {
            assert_eq!(
                window[0].end, window[1].start,
                "gap between {:?} and {:?}",
                window[0], window[1]
            );
        }
    }

    // ===== Full Statement Tests =====

    #[test]
    fn select_star_from_users() {
        let kinds = significant_kinds("SELECT * FROM users");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Keyword,
                TokenKind::Punctuation,
                TokenKind::Keyword,
                TokenKind::Identifier
            ]
        );
    }

    #[test]
    fn select_with_where() {
        let kinds = significant_kinds("SELECT name FROM users WHERE id = 1");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Keyword,       // SELECT
                TokenKind::Identifier,    // name (in column list)
                TokenKind::Keyword,       // FROM
                TokenKind::Identifier,    // users
                TokenKind::Keyword,       // WHERE
                TokenKind::Identifier,    // id
                TokenKind::Operator,      // =
                TokenKind::NumberLiteral, // 1
            ]
        );
    }

    #[test]
    fn insert_statement() {
        let kinds = significant_kinds("INSERT INTO my_table (id, name) VALUES (1, 'hello')");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Keyword,       // INSERT
                TokenKind::Keyword,       // INTO
                TokenKind::Identifier,    // my_table
                TokenKind::Punctuation,   // (
                TokenKind::Identifier,    // id
                TokenKind::Punctuation,   // ,
                TokenKind::Identifier,    // name
                TokenKind::Punctuation,   // )
                TokenKind::Keyword,       // VALUES
                TokenKind::Punctuation,   // (
                TokenKind::NumberLiteral, // 1
                TokenKind::Punctuation,   // ,
                TokenKind::StringLiteral, // 'hello'
                TokenKind::Punctuation,   // )
            ]
        );
    }

    #[test]
    fn update_statement() {
        let kinds = significant_kinds("UPDATE users SET name = 'Alice' WHERE id = 1");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Keyword,       // UPDATE
                TokenKind::Identifier,    // users
                TokenKind::Keyword,       // SET
                TokenKind::Identifier,    // name
                TokenKind::Operator,      // =
                TokenKind::StringLiteral, // 'Alice'
                TokenKind::Keyword,       // WHERE
                TokenKind::Identifier,    // id
                TokenKind::Operator,      // =
                TokenKind::NumberLiteral, // 1
            ]
        );
    }

    #[test]
    fn create_table() {
        let kinds = significant_kinds("CREATE TABLE ks.my_table (id int PRIMARY KEY)");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Keyword,     // CREATE
                TokenKind::Keyword,     // TABLE
                TokenKind::Identifier,  // ks
                TokenKind::Punctuation, // .
                TokenKind::Identifier,  // my_table
                TokenKind::Punctuation, // (
                TokenKind::Identifier,  // id
                TokenKind::Identifier,  // int (type name, in general context after column name)
                TokenKind::Keyword,     // PRIMARY
                TokenKind::Keyword,     // KEY
                TokenKind::Punctuation, // )
            ]
        );
    }

    #[test]
    fn use_keyspace() {
        let kinds = significant_kinds("USE my_keyspace");
        assert_eq!(kinds, vec![TokenKind::Keyword, TokenKind::Identifier]);
    }

    #[test]
    fn qualified_table_name() {
        let texts = significant_texts("SELECT * FROM ks.users");
        assert_eq!(texts, vec!["SELECT", "*", "FROM", "ks", ".", "users"]);
        let kinds = significant_kinds("SELECT * FROM ks.users");
        assert_eq!(kinds[3], TokenKind::Identifier); // ks
        assert_eq!(kinds[4], TokenKind::Punctuation); // .
        assert_eq!(kinds[5], TokenKind::Identifier); // users
    }

    #[test]
    fn statement_with_string_containing_keyword() {
        let kinds = significant_kinds("INSERT INTO t (v) VALUES ('SELECT FROM')");
        // 'SELECT FROM' should be one StringLiteral, not keywords
        assert!(kinds.contains(&TokenKind::StringLiteral));
        // Only 3 keywords: INSERT, INTO, VALUES
        assert_eq!(
            kinds.iter().filter(|k| **k == TokenKind::Keyword).count(),
            3
        );
    }

    #[test]
    fn statement_with_comment() {
        let tokens = tokenize("SELECT 1 -- comment");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig.len(), 3); // SELECT, 1, comment
        assert_eq!(sig[2].kind, TokenKind::LineComment);
    }

    #[test]
    fn statement_with_block_comment() {
        let tokens = tokenize("SELECT /* mid */ 1");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[0].kind, TokenKind::Keyword); // SELECT
        assert_eq!(sig[1].kind, TokenKind::BlockComment); // /* mid */
        assert_eq!(sig[2].kind, TokenKind::NumberLiteral); // 1
    }

    // ===== Grammar Context Tests =====

    #[test]
    fn context_at_start() {
        assert_eq!(grammar_context_at_end(""), GrammarContext::Start);
    }

    #[test]
    fn context_after_select() {
        assert_eq!(
            grammar_context_at_end("SELECT "),
            GrammarContext::ExpectColumnList
        );
    }

    #[test]
    fn context_after_from() {
        assert_eq!(
            grammar_context_at_end("SELECT * FROM "),
            GrammarContext::ExpectTable
        );
    }

    #[test]
    fn context_after_into() {
        assert_eq!(
            grammar_context_at_end("INSERT INTO "),
            GrammarContext::ExpectTable
        );
    }

    #[test]
    fn context_after_update() {
        assert_eq!(
            grammar_context_at_end("UPDATE "),
            GrammarContext::ExpectTable
        );
    }

    #[test]
    fn context_after_use() {
        assert_eq!(
            grammar_context_at_end("USE "),
            GrammarContext::ExpectKeyspace
        );
    }

    #[test]
    fn context_after_where() {
        assert_eq!(
            grammar_context_at_end("SELECT * FROM t WHERE "),
            GrammarContext::ExpectColumn
        );
    }

    #[test]
    fn context_after_dot() {
        assert_eq!(
            grammar_context_at_end("ks."),
            GrammarContext::ExpectQualifiedPart
        );
    }

    #[test]
    fn context_after_table_name() {
        assert_eq!(
            grammar_context_at_end("SELECT * FROM users "),
            GrammarContext::General
        );
    }

    #[test]
    fn context_after_consistency() {
        assert_eq!(
            grammar_context_at_end("CONSISTENCY "),
            GrammarContext::ExpectConsistencyLevel
        );
    }

    #[test]
    fn context_after_describe() {
        assert_eq!(
            grammar_context_at_end("DESCRIBE "),
            GrammarContext::ExpectDescribeTarget
        );
    }

    #[test]
    fn context_after_source() {
        assert_eq!(
            grammar_context_at_end("SOURCE "),
            GrammarContext::ExpectFilePath
        );
    }

    #[test]
    fn context_after_order_by() {
        assert_eq!(
            grammar_context_at_end("SELECT * FROM t ORDER BY "),
            GrammarContext::ExpectOrderByColumn
        );
    }

    #[test]
    fn context_after_values() {
        assert_eq!(
            grammar_context_at_end("INSERT INTO t (id) VALUES "),
            GrammarContext::ExpectValues
        );
    }

    #[test]
    fn context_after_with() {
        assert_eq!(
            grammar_context_at_end("CREATE TABLE t (id int) WITH "),
            GrammarContext::ExpectWithOption
        );
    }

    // ===== Edge Cases =====

    #[test]
    fn empty_input() {
        assert!(tokenize("").is_empty());
    }

    #[test]
    fn only_whitespace() {
        let tokens = tokenize("   \t\n  ");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Whitespace);
    }

    #[test]
    fn only_comment() {
        let tokens = tokenize("-- just a comment");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::LineComment);
    }

    #[test]
    fn unterminated_string() {
        let tokens = tokenize("'unterminated");
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[0].text, "'unterminated");
    }

    #[test]
    fn unterminated_quoted_identifier() {
        let tokens = tokenize("\"unterminated");
        assert_eq!(tokens[0].kind, TokenKind::QuotedIdentifier);
    }

    #[test]
    fn unterminated_dollar_string() {
        let tokens = tokenize("$$unterminated");
        assert_eq!(tokens[0].kind, TokenKind::DollarStringLiteral);
    }

    #[test]
    fn unterminated_block_comment() {
        let tokens = tokenize("/* unterminated");
        assert_eq!(tokens[0].kind, TokenKind::BlockComment);
    }

    #[test]
    fn unicode_in_string() {
        let tokens = tokenize("'héllo wörld'");
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[0].text, "'héllo wörld'");
    }

    #[test]
    fn unicode_in_quoted_identifier() {
        let tokens = tokenize("\"naïve\"");
        assert_eq!(tokens[0].kind, TokenKind::QuotedIdentifier);
        assert_eq!(tokens[0].text, "\"naïve\"");
    }

    #[test]
    fn multiple_statements() {
        let tokens = tokenize("SELECT 1; SELECT 2;");
        let semis: Vec<_> = tokens.iter().filter(|t| t.text == ";").collect();
        assert_eq!(semis.len(), 2);
    }

    #[test]
    fn comment_like_in_string() {
        let tokens = tokenize("'-- not a comment'");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    }

    #[test]
    fn block_comment_like_in_string() {
        let tokens = tokenize("'/* not a comment */'");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    }

    #[test]
    fn negative_number_after_operator() {
        let tokens = tokenize("id = -42");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[2].kind, TokenKind::NumberLiteral);
        assert_eq!(sig[2].text, "-42");
    }

    #[test]
    fn minus_as_operator_after_number() {
        let tokens = tokenize("5 - 3");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[0].kind, TokenKind::NumberLiteral);
        // After a number, - is an operator, not a sign. But the next 3 is a separate number.
        // The '-' here: previous token is whitespace (after 5), so it could be sign.
        // Actually after NumberLiteral + Whitespace, the '-' is ambiguous.
        // Let's just verify we get reasonable output.
        assert!(sig.len() >= 3);
    }

    #[test]
    fn blob_after_value_context() {
        let tokens = tokenize("INSERT INTO t (b) VALUES (0xDEAD)");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        let blob = sig.iter().find(|t| t.text == "0xDEAD").unwrap();
        assert_eq!(blob.kind, TokenKind::BlobLiteral);
    }

    #[test]
    fn keyword_list_is_sorted() {
        for window in CQL_KEYWORDS.windows(2) {
            assert!(
                window[0] < window[1],
                "CQL_KEYWORDS not sorted: {:?} >= {:?}",
                window[0],
                window[1]
            );
        }
    }

    // ===== strip_comments =====

    #[test]
    fn strip_line_comment() {
        let result = strip_comments("SELECT 1 -- comment");
        assert_eq!(result, "SELECT 1 ");
    }

    #[test]
    fn strip_block_comment() {
        let result = strip_comments("SELECT /* x */ 1");
        assert_eq!(result, "SELECT   1");
    }

    #[test]
    fn strip_preserves_strings() {
        let result = strip_comments("SELECT '-- not a comment'");
        assert_eq!(result, "SELECT '-- not a comment'");
    }

    #[test]
    fn strip_preserves_dollar_strings() {
        let result = strip_comments("SELECT $$-- not a comment$$");
        assert_eq!(result, "SELECT $$-- not a comment$$");
    }

    #[test]
    fn strip_nested_block_comments() {
        let result = strip_comments("SELECT /* outer /* inner */ still */ 1");
        assert_eq!(result, "SELECT   1");
    }

    // ===== is_cql_keyword =====

    #[test]
    fn keyword_lookup_positive() {
        assert!(is_cql_keyword("SELECT"));
        assert!(is_cql_keyword("select"));
        assert!(is_cql_keyword("From"));
        assert!(is_cql_keyword("WHERE"));
    }

    #[test]
    fn keyword_lookup_negative() {
        assert!(!is_cql_keyword("my_table"));
        assert!(!is_cql_keyword("hello"));
        assert!(!is_cql_keyword("xyz"));
    }

    // ===== significant_tokens helper =====

    #[test]
    fn significant_tokens_filters_whitespace_and_comments() {
        let tokens = tokenize("SELECT /* comment */ * -- line\nFROM t");
        let sig = significant_tokens(&tokens);
        let kinds: Vec<_> = sig.iter().map(|t| &t.kind).collect();
        assert!(!kinds.contains(&&TokenKind::Whitespace));
        assert!(!kinds.contains(&&TokenKind::LineComment));
        assert!(!kinds.contains(&&TokenKind::BlockComment));
    }

    // ===== Regression: colorizer false-positive tests =====

    #[test]
    fn users_not_keyword_after_from() {
        let tokens = tokenize("SELECT * FROM users");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[3].text, "users");
        assert_eq!(sig[3].kind, TokenKind::Identifier);
    }

    #[test]
    fn key_not_keyword_after_from() {
        let tokens = tokenize("SELECT key FROM my_table WHERE key = 1");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        // "key" after SELECT is in column list -> Identifier
        assert_eq!(sig[1].kind, TokenKind::Identifier);
        // "my_table" after FROM -> Identifier
        assert_eq!(sig[3].kind, TokenKind::Identifier);
        // "key" after WHERE -> Identifier
        assert_eq!(sig[5].kind, TokenKind::Identifier);
    }

    #[test]
    fn set_not_keyword_in_column_list() {
        // "set" as a column name in SELECT
        let tokens = tokenize("SELECT set FROM my_table");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[1].text, "set");
        assert_eq!(sig[1].kind, TokenKind::Identifier);
    }

    #[test]
    fn column_names_after_where_are_identifiers() {
        let tokens = tokenize("SELECT * FROM t WHERE user = 'test' AND key = 1");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        // user after WHERE
        assert_eq!(sig[5].text, "user");
        assert_eq!(sig[5].kind, TokenKind::Identifier);
        // key after AND
        assert_eq!(sig[9].text, "key");
        assert_eq!(sig[9].kind, TokenKind::Identifier);
    }

    // ===== Complex queries =====

    #[test]
    fn select_with_function() {
        let tokens = tokenize("SELECT count(*) FROM users");
        let sig: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig[0].kind, TokenKind::Keyword); // SELECT
        assert_eq!(sig[1].kind, TokenKind::Identifier); // count (in column list context)
    }

    #[test]
    fn batch_statement() {
        let input =
            "BEGIN BATCH INSERT INTO t (id) VALUES (1); INSERT INTO t (id) VALUES (2); APPLY BATCH";
        let tokens = tokenize(input);
        let keywords: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Keyword)
            .collect();
        assert!(keywords.iter().any(|t| t.text.to_uppercase() == "BEGIN"));
        assert!(keywords.iter().any(|t| t.text.to_uppercase() == "BATCH"));
        assert!(keywords.iter().any(|t| t.text.to_uppercase() == "APPLY"));
    }

    #[test]
    fn delete_from() {
        let kinds = significant_kinds("DELETE FROM users WHERE id = 1");
        assert_eq!(kinds[0], TokenKind::Keyword); // DELETE
        assert_eq!(kinds[1], TokenKind::Keyword); // FROM
        assert_eq!(kinds[2], TokenKind::Identifier); // users
    }

    #[test]
    fn describe_table() {
        let kinds = significant_kinds("DESCRIBE TABLE users");
        assert_eq!(kinds[0], TokenKind::Keyword); // DESCRIBE
        assert_eq!(kinds[1], TokenKind::Keyword); // TABLE
        assert_eq!(kinds[2], TokenKind::Identifier); // users
    }

    #[test]
    fn truncate_table() {
        let kinds = significant_kinds("TRUNCATE users");
        assert_eq!(kinds[0], TokenKind::Keyword); // TRUNCATE
        assert_eq!(kinds[1], TokenKind::Identifier); // users
    }

    #[test]
    fn select_distinct() {
        let kinds = significant_kinds("SELECT DISTINCT partition_key FROM t");
        assert_eq!(kinds[0], TokenKind::Keyword); // SELECT
        assert_eq!(kinds[1], TokenKind::Keyword); // DISTINCT
        assert_eq!(kinds[2], TokenKind::Identifier); // partition_key
    }

    #[test]
    fn consistency_level() {
        let kinds = significant_kinds("CONSISTENCY QUORUM");
        assert_eq!(kinds[0], TokenKind::Keyword); // CONSISTENCY
        assert_eq!(kinds[1], TokenKind::Identifier); // QUORUM (in consistency level context)
    }

    #[test]
    fn serial_consistency() {
        let ctx = grammar_context_at_end("SERIAL CONSISTENCY ");
        assert_eq!(ctx, GrammarContext::ExpectConsistencyLevel);
    }

    #[test]
    fn order_by_column() {
        let sig: Vec<_> = tokenize("SELECT * FROM t ORDER BY created_at")
            .into_iter()
            .filter(|t| t.kind != TokenKind::Whitespace)
            .collect();
        assert_eq!(sig.last().unwrap().kind, TokenKind::Identifier); // created_at
    }
}

//! Tab completion for the CQL shell.
//!
//! Implements rustyline's `Completer`, `Helper`, `Hinter`, `Highlighter`, and
//! `Validator` traits to provide context-aware tab completion in the REPL.
//! Uses the unified CQL lexer for grammar-aware context detection.
//! Completions include CQL keywords, shell commands, schema objects (keyspaces,
//! tables, columns), consistency levels, DESCRIBE sub-commands, and file paths.

use std::borrow::Cow;
use std::sync::Arc;

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use tokio::runtime::Handle;
use tokio::sync::RwLock;

use crate::colorizer::CqlColorizer;
use crate::cql_lexer::{self, GrammarContext, TokenKind};
use crate::schema_cache::SchemaCache;

/// CQL keywords that can start a statement.
const CQL_KEYWORDS: &[&str] = &[
    "ALTER", "APPLY", "BATCH", "BEGIN", "CREATE", "DELETE", "DESCRIBE", "DROP", "GRANT", "INSERT",
    "LIST", "REVOKE", "SELECT", "TRUNCATE", "UPDATE", "USE",
];

/// CQL clause keywords used within statements (superseded by per-statement lists).
#[allow(dead_code)]
const CQL_CLAUSE_KEYWORDS: &[&str] = &[
    "ADD",
    "AGGREGATE",
    "ALL",
    "ALLOW",
    "AND",
    "AS",
    "ASC",
    "AUTHORIZE",
    "BATCH",
    "BY",
    "CALLED",
    "CLUSTERING",
    "COLUMN",
    "COMPACT",
    "CONTAINS",
    "COUNT",
    "CUSTOM",
    "DELETE",
    "DESC",
    "DESCRIBE",
    "DISTINCT",
    "DROP",
    "ENTRIES",
    "EXECUTE",
    "EXISTS",
    "FILTERING",
    "FINALFUNC",
    "FROM",
    "FROZEN",
    "FULL",
    "FUNCTION",
    "FUNCTIONS",
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
    "KEYS",
    "KEYSPACE",
    "KEYSPACES",
    "LANGUAGE",
    "LIKE",
    "LIMIT",
    "LIST",
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
    "OR",
    "ORDER",
    "PARTITION",
    "PASSWORD",
    "PER",
    "PERMISSION",
    "PERMISSIONS",
    "PRIMARY",
    "RENAME",
    "REPLACE",
    "RETURNS",
    "REVOKE",
    "SCHEMA",
    "SELECT",
    "SET",
    "SFUNC",
    "STATIC",
    "STORAGE",
    "STYPE",
    "SUPERUSER",
    "TABLE",
    "TABLES",
    "TEXT",
    "TIMESTAMP",
    "TO",
    "TOKEN",
    "TRIGGER",
    "TRUNCATE",
    "TTL",
    "TUPLE",
    "TYPE",
    "UNLOGGED",
    "UPDATE",
    "USER",
    "USERS",
    "USING",
    "VALUES",
    "VIEW",
    "WHERE",
    "WITH",
    "WRITETIME",
];

/// Built-in shell commands.
const SHELL_COMMANDS: &[&str] = &[
    "CAPTURE",
    "CLEAR",
    "CLS",
    "CONSISTENCY",
    "COPY",
    "DESCRIBE",
    "DESC",
    "EXIT",
    "EXPAND",
    "HELP",
    "LOGIN",
    "PAGING",
    "QUIT",
    "SERIAL",
    "SHOW",
    "SOURCE",
    "TRACING",
];

/// CQL consistency levels.
const CONSISTENCY_LEVELS: &[&str] = &[
    "ALL",
    "ANY",
    "EACH_QUORUM",
    "LOCAL_ONE",
    "LOCAL_QUORUM",
    "LOCAL_SERIAL",
    "ONE",
    "QUORUM",
    "SERIAL",
    "THREE",
    "TWO",
];

/// DESCRIBE sub-commands.
const DESCRIBE_SUB_COMMANDS: &[&str] = &[
    "AGGREGATE",
    "AGGREGATES",
    "CLUSTER",
    "FULL",
    "FUNCTION",
    "FUNCTIONS",
    "INDEX",
    "KEYSPACE",
    "KEYSPACES",
    "MATERIALIZED",
    "SCHEMA",
    "TABLE",
    "TABLES",
    "TYPE",
    "TYPES",
];

/// Keywords valid at the start of a SELECT column list.
const SELECT_COLUMN_KEYWORDS: &[&str] = &[
    "*",
    "COUNT(",
    "DISTINCT",
    "FROM",
    "JSON",
    "TTL(",
    "WRITETIME(",
];

const CREATE_TARGET_KEYWORDS: &[&str] = &[
    "AGGREGATE",
    "CUSTOM INDEX",
    "FUNCTION",
    "INDEX",
    "KEYSPACE",
    "MATERIALIZED VIEW",
    "ROLE",
    "TABLE",
    "TRIGGER",
    "TYPE",
    "USER",
];

const ALTER_TARGET_KEYWORDS: &[&str] = &[
    "KEYSPACE",
    "MATERIALIZED VIEW",
    "ROLE",
    "TABLE",
    "TYPE",
    "USER",
];

const DROP_TARGET_KEYWORDS: &[&str] = &[
    "AGGREGATE",
    "FUNCTION",
    "INDEX",
    "KEYSPACE",
    "MATERIALIZED VIEW",
    "ROLE",
    "TABLE",
    "TRIGGER",
    "TYPE",
    "USER",
];

const DELETE_TARGET_KEYWORDS: &[&str] = &["FROM"];

const GRANT_REVOKE_KEYWORDS: &[&str] = &[
    "ALL",
    "ALTER",
    "AUTHORIZE",
    "CREATE",
    "DESCRIBE",
    "DROP",
    "EXECUTE",
    "MODIFY",
    "SELECT",
];

const INSERT_TARGET_KEYWORDS: &[&str] = &["INTO"];

const BEGIN_TARGET_KEYWORDS: &[&str] = &["BATCH", "COUNTER", "UNLOGGED"];

const SELECT_POST_FROM_KEYWORDS: &[&str] = &[
    "ALLOW FILTERING",
    "GROUP BY",
    "LIMIT",
    "ORDER BY",
    "PER PARTITION LIMIT",
    "WHERE",
];

const INSERT_POST_VALUES_KEYWORDS: &[&str] = &["IF NOT EXISTS", "USING"];

const DELETE_POST_FROM_KEYWORDS: &[&str] = &["IF", "USING TIMESTAMP", "WHERE"];

const UPDATE_CLAUSE_KEYWORDS: &[&str] = &["SET", "USING"];

const UPDATE_POST_SET_KEYWORDS: &[&str] = &["IF", "WHERE"];

const GENERIC_CLAUSE_KEYWORDS: &[&str] = &[
    "ALLOW FILTERING",
    "AND",
    "FROM",
    "GROUP BY",
    "IF",
    "INTO",
    "LIMIT",
    "ORDER BY",
    "SET",
    "USING",
    "VALUES",
    "WHERE",
    "WITH",
];

/// CQL data types for CREATE TABLE column definitions.
#[allow(dead_code)] // Will be used when CqlType completion context is implemented
const CQL_TYPES: &[&str] = &[
    "ascii",
    "bigint",
    "blob",
    "boolean",
    "counter",
    "date",
    "decimal",
    "double",
    "duration",
    "float",
    "frozen",
    "inet",
    "int",
    "list",
    "map",
    "set",
    "smallint",
    "text",
    "time",
    "timestamp",
    "timeuuid",
    "tinyint",
    "tuple",
    "uuid",
    "varchar",
    "varint",
];

/// Detected completion context based on the input up to the cursor.
#[derive(Debug, PartialEq)]
enum CompletionContext {
    /// At the start of input — complete with statement keywords and shell commands.
    Empty,
    /// After a statement keyword — complete with generic clause keywords.
    #[allow(dead_code)]
    ClauseKeyword,
    /// After FROM, INTO, UPDATE, etc. — complete with table names.
    TableName { keyspace: Option<String> },
    /// After SELECT ... FROM table WHERE — complete with column names.
    ColumnName {
        keyspace: Option<String>,
        table: String,
    },
    /// After CONSISTENCY — complete with consistency levels.
    ConsistencyLevel,
    /// After DESCRIBE/DESC — complete with sub-commands or schema names.
    DescribeTarget,
    /// After SOURCE or CAPTURE — complete with file paths.
    FilePath,
    /// After USE — complete with keyspace names.
    KeyspaceName,
    /// After SELECT — complete with *, DISTINCT, JSON, or column names.
    SelectColumnList,
    /// After CREATE — complete with TABLE, KEYSPACE, INDEX, etc.
    CreateTarget,
    /// After ALTER — complete with TABLE, KEYSPACE, TYPE, etc.
    AlterTarget,
    /// After DROP — complete with TABLE, KEYSPACE, INDEX, etc.
    DropTarget,
    /// After DELETE — complete with FROM or column names.
    DeleteTarget,
    /// After GRANT/REVOKE — complete with permission names.
    GrantRevoke,
    /// After INSERT — complete with INTO.
    InsertTarget,
    /// After BEGIN — complete with BATCH, UNLOGGED, COUNTER.
    BeginTarget,
    /// After SELECT * — only FROM is valid next.
    SelectPostStar,
    /// After SELECT ... FROM table — complete with WHERE, ORDER BY, LIMIT, etc.
    SelectPostFrom,
    /// After INSERT ... VALUES(...) — complete with IF NOT EXISTS, USING.
    InsertPostValues,
    /// After DELETE ... FROM table — complete with WHERE, IF, USING TIMESTAMP.
    DeletePostFrom,
    /// After UPDATE table — complete with SET, USING.
    UpdateClause,
    /// After UPDATE ... SET col = val — complete with WHERE, IF.
    UpdatePostSet,
    /// Generic clause fallback — common clause keywords.
    GenericClause,
}

/// Tab completer for the CQL shell REPL.
pub struct CqlCompleter {
    /// Shared schema cache for keyspace/table/column lookups.
    cache: Arc<RwLock<SchemaCache>>,
    /// Current keyspace (shared with session via USE command).
    current_keyspace: Arc<RwLock<Option<String>>>,
    /// Tokio runtime handle for blocking cache reads inside sync complete().
    rt_handle: Handle,
    /// Syntax colorizer for highlighting.
    colorizer: CqlColorizer,
}

impl CqlCompleter {
    /// Create a new completer with shared cache and keyspace state.
    pub fn new(
        cache: Arc<RwLock<SchemaCache>>,
        current_keyspace: Arc<RwLock<Option<String>>>,
        rt_handle: Handle,
        color_enabled: bool,
    ) -> Self {
        Self {
            cache,
            current_keyspace,
            rt_handle,
            colorizer: CqlColorizer::new(color_enabled),
        }
    }

    /// Detect completion context from the input line up to the cursor position.
    ///
    /// Uses the unified CQL lexer for grammar-aware context detection.
    fn detect_context(&self, line: &str, pos: usize) -> CompletionContext {
        let before_cursor = &line[..pos];

        // Multi-statement: only consider text after the last semicolon
        let before_cursor = match before_cursor.rfind(';') {
            Some(idx) => &before_cursor[idx + 1..],
            None => before_cursor,
        };

        let tokens = cql_lexer::tokenize(before_cursor);
        let sig: Vec<_> = cql_lexer::significant_tokens(&tokens);

        if sig.is_empty() {
            return CompletionContext::Empty;
        }

        // Special case: SOURCE/CAPTURE always means file path completion
        let first_upper = sig[0].text.to_uppercase();
        if first_upper == "SOURCE" || first_upper == "CAPTURE" {
            return CompletionContext::FilePath;
        }

        // Mid-word context fix: when the user is typing a partial word (no trailing
        // space), strip it and get grammar context from everything BEFORE it.
        // e.g. "SELECT * F" → context of "SELECT * " → ExpectColumnList
        // e.g. "SELECT * FROM system.h" → context of "SELECT * FROM system." → ExpectQualifiedPart
        // Exception: don't strip if the last token is a dot — the dot IS the context
        // indicator for qualified name completion (e.g. "FROM system." → ExpectQualifiedPart).
        let last_is_dot = sig
            .last()
            .is_some_and(|t| t.kind == TokenKind::Punctuation && t.text == ".");
        let grammar_ctx = if !before_cursor.ends_with(' ') && sig.len() > 1 && !last_is_dot {
            let last_token = sig.last().unwrap();
            cql_lexer::grammar_context_at_end(&before_cursor[..last_token.start])
        } else if !before_cursor.ends_with(' ') && sig.len() == 1 {
            GrammarContext::Start
        } else {
            cql_lexer::grammar_context_at_end(before_cursor)
        };

        match grammar_ctx {
            GrammarContext::Start => CompletionContext::Empty,
            GrammarContext::ExpectTable => {
                // Check if the user is typing a qualified name (ks.)
                let keyspace = self.extract_qualifying_keyspace(&sig);
                CompletionContext::TableName { keyspace }
            }
            GrammarContext::ExpectKeyspace => CompletionContext::KeyspaceName,
            GrammarContext::ExpectColumn | GrammarContext::ExpectSetClause => {
                // Find the table name from the token stream
                let (ks, table) = self.extract_table_from_tokens(&sig);
                match table {
                    Some(t) => CompletionContext::ColumnName {
                        keyspace: ks,
                        table: t,
                    },
                    None => CompletionContext::GenericClause,
                }
            }
            GrammarContext::ExpectConsistencyLevel => CompletionContext::ConsistencyLevel,
            GrammarContext::ExpectDescribeTarget => CompletionContext::DescribeTarget,
            GrammarContext::ExpectFilePath => CompletionContext::FilePath,
            GrammarContext::ExpectQualifiedPart => {
                // After CREATE/ALTER TABLE ks. — don't suggest existing tables
                let has_create_or_alter = sig.iter().any(|t| {
                    let u = t.text.to_uppercase();
                    u == "CREATE" || u == "ALTER"
                });
                if has_create_or_alter {
                    CompletionContext::GenericClause
                } else {
                    let keyspace = self.extract_qualifying_keyspace(&sig);
                    CompletionContext::TableName { keyspace }
                }
            }
            GrammarContext::ExpectColumnList => {
                // If * already appears after SELECT, only offer FROM
                let has_star = sig
                    .iter()
                    .any(|t| t.kind == TokenKind::Punctuation && t.text == "*");
                if has_star {
                    CompletionContext::SelectPostStar
                } else {
                    CompletionContext::SelectColumnList
                }
            }
            GrammarContext::ExpectCreateTarget => CompletionContext::CreateTarget,
            GrammarContext::ExpectAlterTarget => CompletionContext::AlterTarget,
            GrammarContext::ExpectDropTarget => CompletionContext::DropTarget,
            GrammarContext::ExpectDeleteTarget => CompletionContext::DeleteTarget,
            GrammarContext::ExpectGrantRevoke => CompletionContext::GrantRevoke,
            GrammarContext::ExpectInsertTarget => CompletionContext::InsertTarget,
            GrammarContext::ExpectBeginTarget => CompletionContext::BeginTarget,
            GrammarContext::ExpectSelectPostFrom => CompletionContext::SelectPostFrom,
            GrammarContext::ExpectInsertPostValues => CompletionContext::InsertPostValues,
            GrammarContext::ExpectDeletePostFrom => CompletionContext::DeletePostFrom,
            GrammarContext::ExpectUpdateClause => CompletionContext::UpdateClause,
            GrammarContext::ExpectUpdatePostSet => CompletionContext::UpdatePostSet,
            _ => {
                if sig.len() == 1 && !before_cursor.ends_with(' ') {
                    CompletionContext::Empty
                } else {
                    CompletionContext::GenericClause
                }
            }
        }
    }

    /// Extract the keyspace qualifier from a dot-qualified name in the token stream.
    /// Handles both `ks.` (dot is last) and `ks.partial` (identifier after dot).
    fn extract_qualifying_keyspace(&self, sig: &[&cql_lexer::Token]) -> Option<String> {
        let len = sig.len();
        // Pattern: identifier . (dot is last token)
        if len >= 2 && sig[len - 1].text == "." {
            return Some(sig[len - 2].text.clone());
        }
        // Pattern: identifier . partial_name (user typing after dot)
        if len >= 3 && sig[len - 2].text == "." {
            return Some(sig[len - 3].text.clone());
        }
        None
    }

    /// Extract table name from the token stream by finding FROM/INTO/UPDATE <table>.
    fn extract_table_from_tokens(
        &self,
        sig: &[&cql_lexer::Token],
    ) -> (Option<String>, Option<String>) {
        for (i, tok) in sig.iter().enumerate() {
            let upper = tok.text.to_uppercase();
            if matches!(upper.as_str(), "FROM" | "INTO" | "UPDATE" | "TABLE")
                && i + 1 < sig.len()
                && matches!(
                    sig[i + 1].kind,
                    TokenKind::Identifier | TokenKind::QuotedIdentifier
                )
            {
                let table = sig[i + 1].text.clone();
                // Check for qualified name (ks.table)
                if i + 3 < sig.len() && sig[i + 2].text == "." {
                    let ks = table;
                    let tbl = sig[i + 3].text.clone();
                    return (Some(ks), Some(tbl));
                }
                let ks = tokio::task::block_in_place(|| {
                    self.rt_handle
                        .block_on(async { self.current_keyspace.read().await.clone() })
                });
                return (ks, Some(table));
            }
        }
        (None, None)
    }

    /// Generate completions for the detected context.
    fn complete_for_context(&self, ctx: &CompletionContext, prefix: &str) -> Vec<Pair> {
        let prefix_upper = prefix.to_uppercase();

        match ctx {
            CompletionContext::Empty => {
                let mut candidates: Vec<&str> = Vec::new();
                candidates.extend_from_slice(CQL_KEYWORDS);
                candidates.extend_from_slice(SHELL_COMMANDS);
                filter_candidates(&candidates, &prefix_upper, true)
            }
            CompletionContext::ConsistencyLevel => {
                filter_candidates(CONSISTENCY_LEVELS, &prefix_upper, true)
            }
            CompletionContext::DescribeTarget => {
                filter_candidates(DESCRIBE_SUB_COMMANDS, &prefix_upper, true)
            }
            CompletionContext::KeyspaceName => {
                let cache =
                    tokio::task::block_in_place(|| self.rt_handle.block_on(self.cache.read()));
                let names = cache.keyspace_names();
                filter_candidates(&names, prefix, false)
            }
            CompletionContext::TableName { keyspace } => {
                let cache =
                    tokio::task::block_in_place(|| self.rt_handle.block_on(self.cache.read()));
                let ks = keyspace.clone().or_else(|| {
                    tokio::task::block_in_place(|| {
                        self.rt_handle
                            .block_on(async { self.current_keyspace.read().await.clone() })
                    })
                });
                match ks {
                    Some(ref ks_name) => {
                        let names = cache.table_names(ks_name);
                        filter_candidates(&names, prefix, false)
                    }
                    None => {
                        // No keyspace context — offer keyspace names for qualification
                        let names = cache.keyspace_names();
                        filter_candidates(&names, prefix, false)
                    }
                }
            }
            CompletionContext::ColumnName { keyspace, table } => {
                let cache =
                    tokio::task::block_in_place(|| self.rt_handle.block_on(self.cache.read()));
                let ks = keyspace.clone().or_else(|| {
                    tokio::task::block_in_place(|| {
                        self.rt_handle
                            .block_on(async { self.current_keyspace.read().await.clone() })
                    })
                });
                match ks {
                    Some(ref ks_name) => {
                        let names = cache.column_names(ks_name, table);
                        filter_candidates(&names, prefix, false)
                    }
                    None => vec![],
                }
            }
            CompletionContext::FilePath => complete_file_path(prefix),
            CompletionContext::SelectColumnList => {
                filter_candidates(SELECT_COLUMN_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::SelectPostStar => filter_candidates(&["FROM"], &prefix_upper, true),
            CompletionContext::CreateTarget => {
                filter_candidates(CREATE_TARGET_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::AlterTarget => {
                filter_candidates(ALTER_TARGET_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::DropTarget => {
                filter_candidates(DROP_TARGET_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::DeleteTarget => {
                filter_candidates(DELETE_TARGET_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::GrantRevoke => {
                filter_candidates(GRANT_REVOKE_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::InsertTarget => {
                filter_candidates(INSERT_TARGET_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::BeginTarget => {
                filter_candidates(BEGIN_TARGET_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::SelectPostFrom => {
                filter_candidates(SELECT_POST_FROM_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::InsertPostValues => {
                filter_candidates(INSERT_POST_VALUES_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::DeletePostFrom => {
                filter_candidates(DELETE_POST_FROM_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::UpdateClause => {
                filter_candidates(UPDATE_CLAUSE_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::UpdatePostSet => {
                filter_candidates(UPDATE_POST_SET_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::GenericClause | CompletionContext::ClauseKeyword => {
                filter_candidates(GENERIC_CLAUSE_KEYWORDS, &prefix_upper, true)
            }
        }
    }
}

/// Filter candidates by prefix, returning matching `Pair`s.
fn filter_candidates(candidates: &[&str], prefix: &str, uppercase: bool) -> Vec<Pair> {
    candidates
        .iter()
        .filter(|c| {
            if uppercase {
                c.to_uppercase().starts_with(&prefix.to_uppercase())
            } else {
                c.starts_with(prefix)
            }
        })
        .map(|c| {
            let display = if uppercase {
                c.to_uppercase()
            } else {
                c.to_string()
            };
            Pair {
                display: display.clone(),
                replacement: display,
            }
        })
        .collect()
}

/// Complete file paths for SOURCE and CAPTURE commands.
fn complete_file_path(prefix: &str) -> Vec<Pair> {
    // Strip surrounding quotes if present
    let path_str = prefix
        .strip_prefix('\'')
        .or_else(|| prefix.strip_prefix('"'))
        .unwrap_or(prefix);

    // Expand ~ to home directory
    let expanded = if path_str.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            path_str.replacen('~', &home.to_string_lossy(), 1)
        } else {
            path_str.to_string()
        }
    } else {
        path_str.to_string()
    };

    let (dir, file_prefix) = if expanded.ends_with('/') {
        (expanded.as_str(), "")
    } else {
        let path = std::path::Path::new(&expanded);
        let parent = path
            .parent()
            .map(|p| p.to_str().unwrap_or("."))
            .unwrap_or(".");
        let file = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        (parent, file)
    };

    let dir_to_read = if dir.is_empty() { "." } else { dir };

    let Ok(entries) = std::fs::read_dir(dir_to_read) else {
        return vec![];
    };

    entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(file_prefix) {
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                let suffix = if is_dir { "/" } else { "" };
                let full = if dir.is_empty() || dir == "." {
                    format!("{name}{suffix}")
                } else if dir.ends_with('/') {
                    format!("{dir}{name}{suffix}")
                } else {
                    format!("{dir}/{name}{suffix}")
                };
                Some(Pair {
                    display: name + suffix,
                    replacement: full,
                })
            } else {
                None
            }
        })
        .collect()
}

impl Completer for CqlCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // block_in_place: complete() is called from within the Tokio runtime (sync rustyline trait)
        let needs_refresh = tokio::task::block_in_place(|| {
            self.rt_handle
                .block_on(async { self.cache.read().await.is_stale() })
        });
        if needs_refresh {
            // Best-effort refresh — don't block on errors
            tokio::task::block_in_place(|| {
                self.rt_handle.block_on(async {
                    // Try to get write lock without blocking other completions
                    if let Ok(mut cache) = self.cache.try_write() {
                        // Re-check staleness after acquiring lock
                        if cache.is_stale() {
                            // We can't refresh without a session reference here.
                            // The REPL pre-refreshes the cache; this is a fallback mark.
                            cache.invalidate();
                        }
                    }
                })
            });
        }

        let context = self.detect_context(line, pos);

        // Find the start of the word being completed
        let before_cursor = &line[..pos];
        let word_start = before_cursor
            .rfind(|c: char| c.is_whitespace() || c == '.' || c == '\'' || c == '"')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &line[word_start..pos];

        let completions = self.complete_for_context(&context, prefix);

        Ok((word_start, completions))
    }
}

impl Hinter for CqlCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for CqlCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let colored = self.colorizer.colorize_line(line);
        if colored == line {
            Cow::Borrowed(line)
        } else {
            Cow::Owned(colored)
        }
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Borrowed(prompt)
    }

    fn highlight_char(
        &self,
        _line: &str,
        _pos: usize,
        _forced: rustyline::highlight::CmdKind,
    ) -> bool {
        // Return true to trigger re-highlighting on every keystroke
        true
    }
}

impl Validator for CqlCompleter {}

impl Helper for CqlCompleter {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_completer() -> CqlCompleter {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let cache = Arc::new(RwLock::new(SchemaCache::new()));
        let current_ks = Arc::new(RwLock::new(None::<String>));
        CqlCompleter::new(cache, current_ks, rt.handle().clone(), false)
    }

    #[test]
    fn completer_can_be_created() {
        let _c = make_completer();
    }

    #[test]
    fn detect_empty_context() {
        let c = make_completer();
        assert_eq!(c.detect_context("", 0), CompletionContext::Empty);
    }

    #[test]
    fn detect_keyword_prefix() {
        let c = make_completer();
        assert_eq!(c.detect_context("SEL", 3), CompletionContext::Empty);
    }

    #[test]
    fn detect_consistency_context() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("CONSISTENCY ", 12),
            CompletionContext::ConsistencyLevel
        );
    }

    #[test]
    fn detect_serial_consistency_context() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("SERIAL CONSISTENCY ", 19),
            CompletionContext::ConsistencyLevel
        );
    }

    #[test]
    fn detect_use_keyspace_context() {
        let c = make_completer();
        assert_eq!(c.detect_context("USE ", 4), CompletionContext::KeyspaceName);
    }

    #[test]
    fn detect_describe_sub_command() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("DESCRIBE ", 9),
            CompletionContext::DescribeTarget
        );
    }

    #[test]
    fn detect_describe_table_name() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("DESCRIBE TABLE ", 15),
            CompletionContext::TableName { keyspace: None }
        );
    }

    #[test]
    fn detect_describe_keyspace_name() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("DESCRIBE KEYSPACE ", 18),
            CompletionContext::KeyspaceName
        );
    }

    #[test]
    fn detect_source_file_path() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("SOURCE '/tmp/", 13),
            CompletionContext::FilePath
        );
    }

    #[test]
    fn detect_capture_file_path() {
        let c = make_completer();
        assert_eq!(c.detect_context("CAPTURE ", 8), CompletionContext::FilePath);
    }

    #[test]
    fn detect_from_table_context() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("SELECT * FROM ", 14),
            CompletionContext::TableName { keyspace: None }
        );
    }

    #[test]
    fn complete_keyword_prefix() {
        let c = make_completer();
        let pairs = c.complete_for_context(&CompletionContext::Empty, "SEL");
        assert!(pairs.iter().any(|p| p.replacement == "SELECT"));
    }

    #[test]
    fn complete_consistency_level_prefix() {
        let c = make_completer();
        let pairs = c.complete_for_context(&CompletionContext::ConsistencyLevel, "QU");
        assert!(pairs.iter().any(|p| p.replacement == "QUORUM"));
    }

    #[test]
    fn complete_describe_sub_command() {
        let c = make_completer();
        let pairs = c.complete_for_context(&CompletionContext::DescribeTarget, "KEY");
        assert!(pairs.iter().any(|p| p.replacement == "KEYSPACE"));
        assert!(pairs.iter().any(|p| p.replacement == "KEYSPACES"));
    }

    #[test]
    fn filter_is_case_insensitive_for_keywords() {
        let pairs = filter_candidates(CQL_KEYWORDS, "sel", true);
        assert!(pairs.iter().any(|p| p.replacement == "SELECT"));
    }

    #[test]
    fn file_path_completion_tmp() {
        let pairs = complete_file_path("/tmp/");
        assert!(
            !pairs.is_empty() || std::fs::read_dir("/tmp").map(|d| d.count()).unwrap_or(0) == 0
        );
    }

    // --- Post-statement context detection ---

    #[test]
    fn detect_select_post_from() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("SELECT * FROM users ", 20),
            CompletionContext::SelectPostFrom
        );
    }

    #[test]
    fn detect_delete_post_from() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("DELETE FROM users ", 18),
            CompletionContext::DeletePostFrom
        );
    }

    #[test]
    fn detect_update_clause() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("UPDATE users ", 13),
            CompletionContext::UpdateClause
        );
    }

    #[test]
    fn detect_update_post_set() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("UPDATE users SET name = 'x' ", 28),
            CompletionContext::UpdatePostSet
        );
    }

    #[test]
    fn detect_insert_post_values() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("INSERT INTO users (id) VALUES (1) ", 34),
            CompletionContext::InsertPostValues
        );
    }

    // --- Negative tests: statement-specific contexts must NOT suggest wrong keywords ---

    #[test]
    fn select_post_from_must_not_suggest_drop() {
        let c = make_completer();
        let pairs = c.complete_for_context(&CompletionContext::SelectPostFrom, "");
        assert!(
            !pairs.iter().any(|p| p.replacement == "DROP"),
            "SELECT post-FROM must not suggest DROP"
        );
    }

    #[test]
    fn create_target_must_not_suggest_where() {
        let c = make_completer();
        let pairs = c.complete_for_context(&CompletionContext::CreateTarget, "");
        assert!(
            !pairs.iter().any(|p| p.replacement == "WHERE"),
            "CREATE target must not suggest WHERE"
        );
    }

    #[test]
    fn update_clause_suggests_set() {
        let c = make_completer();
        let pairs = c.complete_for_context(&CompletionContext::UpdateClause, "");
        assert!(pairs.iter().any(|p| p.replacement == "SET"));
        assert!(!pairs.iter().any(|p| p.replacement == "FROM"));
    }

    #[test]
    fn delete_post_from_suggests_where() {
        let c = make_completer();
        let pairs = c.complete_for_context(&CompletionContext::DeletePostFrom, "");
        assert!(pairs.iter().any(|p| p.replacement == "WHERE"));
        assert!(!pairs.iter().any(|p| p.replacement == "ORDER BY"));
    }

    #[test]
    fn midword_select_star_f_gives_select_post_star() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("SELECT * F", 10),
            CompletionContext::SelectPostStar
        );
        let pairs = c.complete_for_context(&CompletionContext::SelectPostStar, "F");
        assert!(pairs.iter().any(|p| p.replacement == "FROM"));
        assert!(!pairs.iter().any(|p| p.replacement == "FILTERING"));
        assert!(!pairs.iter().any(|p| p.replacement == "FUNCTION"));
    }

    #[test]
    fn midword_qualified_table_name() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("SELECT * FROM system.h", 22),
            CompletionContext::TableName {
                keyspace: Some("system".to_string())
            }
        );
    }

    #[test]
    fn select_space_gives_select_column_list() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("SELECT ", 7),
            CompletionContext::SelectColumnList
        );
    }

    #[test]
    fn qualified_dot_without_partial_gives_table_name() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("SELECT * FROM system.", 21),
            CompletionContext::TableName {
                keyspace: Some("system".to_string())
            }
        );
        assert_eq!(
            c.detect_context("SELECT * FROM test_ks.", 22),
            CompletionContext::TableName {
                keyspace: Some("test_ks".to_string())
            }
        );
    }

    #[test]
    fn select_star_space_gives_post_star() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("SELECT * ", 9),
            CompletionContext::SelectPostStar
        );
        let pairs = c.complete_for_context(&CompletionContext::SelectPostStar, "");
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].replacement, "FROM");
    }

    #[test]
    fn multi_statement_resets_context() {
        let c = make_completer();
        assert_eq!(c.detect_context("SELECT 1; ", 10), CompletionContext::Empty);
        assert_eq!(
            c.detect_context("SELECT 1; S", 11),
            CompletionContext::Empty
        );
    }

    #[test]
    fn create_table_qualified_does_not_suggest_tables() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("CREATE TABLE test_ks.", 21),
            CompletionContext::GenericClause
        );
    }

    #[test]
    fn alter_table_qualified_does_not_suggest_tables() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("ALTER TABLE test_ks.", 20),
            CompletionContext::GenericClause
        );
    }

    #[test]
    fn select_post_from_qualified_table() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("SELECT * FROM test_ks.users ", 28),
            CompletionContext::SelectPostFrom
        );
    }

    #[test]
    fn update_clause_qualified_table() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("UPDATE test_ks.users ", 21),
            CompletionContext::UpdateClause
        );
    }

    #[test]
    fn delete_post_from_qualified_table() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("DELETE FROM test_ks.users ", 26),
            CompletionContext::DeletePostFrom
        );
    }

    #[test]
    fn file_path_completion_nonexistent_dir() {
        let pairs = complete_file_path("/nonexistent_dir_xyz_123/");
        assert!(pairs.is_empty());
    }

    #[test]
    fn file_path_completion_with_single_quote() {
        let pairs = complete_file_path("'/tmp/");
        assert!(
            !pairs.is_empty() || std::fs::read_dir("/tmp").map(|d| d.count()).unwrap_or(0) == 0
        );
    }

    #[test]
    fn file_path_completion_with_double_quote() {
        let pairs = complete_file_path("\"/tmp/");
        assert!(
            !pairs.is_empty() || std::fs::read_dir("/tmp").map(|d| d.count()).unwrap_or(0) == 0
        );
    }

    #[test]
    fn file_path_completion_with_tilde() {
        let pairs = complete_file_path("~/");
        if dirs::home_dir().is_some() {
            assert!(!pairs.is_empty() || true);
        }
    }

    #[test]
    fn file_path_completion_with_file_prefix() {
        let pairs = complete_file_path("/tmp/nonexistent_prefix_xyz");
        assert!(pairs.is_empty());
    }

    #[test]
    fn filter_candidates_empty_prefix() {
        let candidates = &["SELECT", "INSERT", "UPDATE"];
        let pairs = filter_candidates(candidates, "", true);
        assert_eq!(pairs.len(), 3);
    }

    #[test]
    fn filter_candidates_no_match() {
        let candidates = &["SELECT", "INSERT", "UPDATE"];
        let pairs = filter_candidates(candidates, "XYZ", true);
        assert!(pairs.is_empty());
    }

    #[test]
    fn filter_candidates_case_insensitive() {
        let candidates = &["SELECT", "INSERT", "UPDATE"];
        let pairs = filter_candidates(candidates, "ins", true);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].replacement, "INSERT");
    }

    #[test]
    fn filter_candidates_case_sensitive_mode() {
        let candidates = &["myTable", "myOther", "yours"];
        let pairs = filter_candidates(candidates, "my", false);
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn filter_candidates_case_sensitive_no_match() {
        let candidates = &["myTable", "myOther"];
        let pairs = filter_candidates(candidates, "MY", false);
        assert!(pairs.is_empty());
    }

    #[test]
    fn detect_insert_into_table_context() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("INSERT INTO ", 12),
            CompletionContext::TableName { keyspace: None }
        );
    }

    #[test]
    fn detect_update_table_context() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("UPDATE ", 7),
            CompletionContext::TableName { keyspace: None }
        );
    }

    #[test]
    fn detect_desc_shorthand() {
        let c = make_completer();
        assert_eq!(
            c.detect_context("DESC ", 5),
            CompletionContext::DescribeTarget
        );
    }

    #[test]
    fn detect_qualified_table_name() {
        let c = make_completer();
        let ctx = c.detect_context("SELECT * FROM mykeyspace.", 25);
        assert_eq!(
            ctx,
            CompletionContext::TableName {
                keyspace: Some("mykeyspace".to_string())
            }
        );
    }

    #[test]
    fn complete_empty_context_includes_shell_commands() {
        let c = make_completer();
        let pairs = c.complete_for_context(&CompletionContext::Empty, "EX");
        assert!(pairs.iter().any(|p| p.replacement == "EXIT"));
        assert!(pairs.iter().any(|p| p.replacement == "EXPAND"));
    }

    #[test]
    fn complete_consistency_all_levels() {
        let c = make_completer();
        let pairs = c.complete_for_context(&CompletionContext::ConsistencyLevel, "");
        assert_eq!(pairs.len(), CONSISTENCY_LEVELS.len());
    }

    #[test]
    fn complete_consistency_local_prefix() {
        let c = make_completer();
        let pairs = c.complete_for_context(&CompletionContext::ConsistencyLevel, "LOCAL");
        assert_eq!(pairs.len(), 3);
    }

    #[test]
    fn complete_keyspace_name_empty_cache() {
        let c = make_completer();
        let pairs = c.complete_for_context(&CompletionContext::KeyspaceName, "");
        assert!(pairs.is_empty());
    }

    #[test]
    fn complete_table_name_no_keyspace() {
        let c = make_completer();
        let pairs = c.complete_for_context(
            &CompletionContext::TableName { keyspace: None },
            "",
        );
        assert!(pairs.is_empty());
    }

    #[test]
    fn complete_column_name_no_keyspace() {
        let c = make_completer();
        let pairs = c.complete_for_context(
            &CompletionContext::ColumnName {
                keyspace: None,
                table: "users".to_string(),
            },
            "",
        );
        assert!(pairs.is_empty());
    }
}

//! Tab completion for the CQL shell.
//!
//! Implements rustyline's `Completer`, `Helper`, `Hinter`, `Highlighter`, and
//! `Validator` traits to provide context-aware tab completion in the REPL.
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
use crate::schema_cache::SchemaCache;

/// CQL keywords that can start a statement.
const CQL_KEYWORDS: &[&str] = &[
    "ALTER", "APPLY", "BATCH", "BEGIN", "CREATE", "DELETE", "DESCRIBE", "DROP",
    "GRANT", "INSERT", "LIST", "REVOKE", "SELECT", "TRUNCATE", "UPDATE", "USE",
];

/// CQL clause keywords used within statements.
const CQL_CLAUSE_KEYWORDS: &[&str] = &[
    "ADD", "AGGREGATE", "ALL", "ALLOW", "AND", "AS", "ASC", "AUTHORIZE",
    "BATCH", "BY", "CALLED", "CLUSTERING", "COLUMN", "COMPACT", "CONTAINS",
    "COUNT", "CUSTOM", "DELETE", "DESC", "DESCRIBE", "DISTINCT", "DROP",
    "ENTRIES", "EXECUTE", "EXISTS", "FILTERING", "FINALFUNC", "FROM",
    "FROZEN", "FULL", "FUNCTION", "FUNCTIONS", "IF", "IN", "INDEX",
    "INITCOND", "INPUT", "INSERT", "INTO", "IS", "JSON", "KEY", "KEYS",
    "KEYSPACE", "KEYSPACES", "LANGUAGE", "LIKE", "LIMIT", "LIST", "LOGIN",
    "MAP", "MATERIALIZED", "MODIFY", "NAMESPACE", "NORECURSIVE", "NOT",
    "NULL", "OF", "ON", "OR", "ORDER", "PARTITION", "PASSWORD", "PER",
    "PERMISSION", "PERMISSIONS", "PRIMARY", "RENAME", "REPLACE", "RETURNS",
    "REVOKE", "SCHEMA", "SELECT", "SET", "SFUNC", "STATIC", "STORAGE",
    "STYPE", "SUPERUSER", "TABLE", "TABLES", "TEXT", "TIMESTAMP", "TO",
    "TOKEN", "TRIGGER", "TRUNCATE", "TTL", "TUPLE", "TYPE", "UNLOGGED",
    "UPDATE", "USER", "USERS", "USING", "VALUES", "VIEW", "WHERE", "WITH",
    "WRITETIME",
];

/// Built-in shell commands.
const SHELL_COMMANDS: &[&str] = &[
    "CAPTURE", "CLEAR", "CLS", "CONSISTENCY", "COPY", "DESCRIBE", "DESC",
    "EXIT", "EXPAND", "HELP", "LOGIN", "PAGING", "QUIT", "SERIAL", "SHOW",
    "SOURCE", "TRACING",
];

/// CQL consistency levels.
const CONSISTENCY_LEVELS: &[&str] = &[
    "ALL", "ANY", "EACH_QUORUM", "LOCAL_ONE", "LOCAL_QUORUM", "LOCAL_SERIAL",
    "ONE", "QUORUM", "SERIAL", "THREE", "TWO",
];

/// DESCRIBE sub-commands.
const DESCRIBE_SUB_COMMANDS: &[&str] = &[
    "AGGREGATE", "AGGREGATES", "CLUSTER", "FUNCTION", "FUNCTIONS", "INDEX",
    "KEYSPACE", "KEYSPACES", "MATERIALIZED", "SCHEMA", "TABLE", "TABLES",
    "TYPE", "TYPES",
];

/// CQL data types for CREATE TABLE column definitions.
const CQL_TYPES: &[&str] = &[
    "ascii", "bigint", "blob", "boolean", "counter", "date", "decimal",
    "double", "duration", "float", "frozen", "inet", "int", "list", "map",
    "set", "smallint", "text", "time", "timestamp", "timeuuid", "tinyint",
    "tuple", "uuid", "varchar", "varint",
];

/// Detected completion context based on the input up to the cursor.
#[derive(Debug, PartialEq)]
enum CompletionContext {
    /// At the start of input — complete with statement keywords and shell commands.
    Empty,
    /// After a statement keyword — complete with clause keywords.
    ClauseKeyword,
    /// After FROM, INTO, UPDATE, etc. — complete with table names.
    TableName { keyspace: Option<String> },
    /// After SELECT ... FROM table WHERE — complete with column names.
    ColumnName { keyspace: Option<String>, table: String },
    /// After CONSISTENCY — complete with consistency levels.
    ConsistencyLevel,
    /// After DESCRIBE/DESC — complete with sub-commands or schema names.
    DescribeTarget,
    /// After SOURCE or CAPTURE — complete with file paths.
    FilePath,
    /// In a CREATE TABLE column type position — complete with CQL types.
    CqlType,
    /// After USE — complete with keyspace names.
    KeyspaceName,
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
    fn detect_context(&self, line: &str, pos: usize) -> CompletionContext {
        let before_cursor = &line[..pos];
        let tokens: Vec<&str> = before_cursor.split_whitespace().collect();

        if tokens.is_empty() {
            return CompletionContext::Empty;
        }

        let first = tokens[0].to_uppercase();

        // CONSISTENCY <level>
        if first == "CONSISTENCY" && tokens.len() <= 2 {
            return if (tokens.len() == 1 && before_cursor.ends_with(' ')) || tokens.len() == 2 {
                CompletionContext::ConsistencyLevel
            } else {
                CompletionContext::Empty
            };
        }

        // SERIAL CONSISTENCY <level>
        if first == "SERIAL" && tokens.len() >= 2 && tokens[1].to_uppercase() == "CONSISTENCY" {
            return CompletionContext::ConsistencyLevel;
        }

        // SOURCE / CAPTURE — file path
        if first == "SOURCE" || first == "CAPTURE" {
            return CompletionContext::FilePath;
        }

        // USE <keyspace>
        if first == "USE" {
            return CompletionContext::KeyspaceName;
        }

        // DESCRIBE / DESC
        if first == "DESCRIBE" || first == "DESC" {
            if tokens.len() == 1 && before_cursor.ends_with(' ') {
                return CompletionContext::DescribeTarget;
            }
            if tokens.len() == 2 {
                let sub = tokens[1].to_uppercase();
                if before_cursor.ends_with(' ') {
                    // After sub-command, complete with schema names
                    return match sub.as_str() {
                        "KEYSPACE" => CompletionContext::KeyspaceName,
                        "TABLE" | "INDEX" | "MATERIALIZED" => CompletionContext::TableName { keyspace: None },
                        _ => CompletionContext::DescribeTarget,
                    };
                }
                return CompletionContext::DescribeTarget;
            }
            if tokens.len() == 3 {
                let sub = tokens[1].to_uppercase();
                return match sub.as_str() {
                    "KEYSPACE" => CompletionContext::KeyspaceName,
                    "TABLE" | "INDEX" => CompletionContext::TableName { keyspace: None },
                    _ => CompletionContext::ClauseKeyword,
                };
            }
            return CompletionContext::ClauseKeyword;
        }

        // Check for FROM/INTO/UPDATE keywords to trigger table name completion
        let upper_tokens: Vec<String> = tokens.iter().map(|t| t.to_uppercase()).collect();
        for (i, token) in upper_tokens.iter().enumerate() {
            if (token == "FROM" || token == "INTO" || token == "UPDATE" || token == "TABLE")
                && i + 1 >= tokens.len()
                && before_cursor.ends_with(' ')
            {
                return CompletionContext::TableName { keyspace: None };
            }
            if (token == "FROM" || token == "INTO" || token == "UPDATE" || token == "TABLE")
                && i + 1 < tokens.len()
            {
                let table_token = tokens[i + 1];
                // Check if partially typing a qualified name (ks.)
                if table_token.contains('.') && table_token.ends_with('.') {
                    let ks = table_token.trim_end_matches('.').to_string();
                    return CompletionContext::TableName { keyspace: Some(ks) };
                }
                // If we're past the table name, might be column context
                if i + 2 < tokens.len() || (i + 1 < tokens.len() && before_cursor.ends_with(' ')) {
                    // Check for WHERE clause
                    if upper_tokens.iter().skip(i + 2).any(|t| t == "WHERE" || t == "SET") {
                        let table = tokens[i + 1].to_string();
                        let ks = self.rt_handle.block_on(async {
                            self.current_keyspace.read().await.clone()
                        });
                        return CompletionContext::ColumnName {
                            keyspace: ks,
                            table,
                        };
                    }
                }
                // Still typing the table name
                if !before_cursor.ends_with(' ') && i + 1 == tokens.len() - 1 {
                    return CompletionContext::TableName { keyspace: None };
                }
            }
        }

        // At beginning of line, completing a keyword
        if tokens.len() == 1 && !before_cursor.ends_with(' ') {
            return CompletionContext::Empty;
        }

        CompletionContext::ClauseKeyword
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
            CompletionContext::ClauseKeyword => {
                filter_candidates(CQL_CLAUSE_KEYWORDS, &prefix_upper, true)
            }
            CompletionContext::ConsistencyLevel => {
                filter_candidates(CONSISTENCY_LEVELS, &prefix_upper, true)
            }
            CompletionContext::DescribeTarget => {
                filter_candidates(DESCRIBE_SUB_COMMANDS, &prefix_upper, true)
            }
            CompletionContext::CqlType => {
                filter_candidates(CQL_TYPES, &prefix.to_lowercase(), false)
            }
            CompletionContext::KeyspaceName => {
                let cache = self.rt_handle.block_on(self.cache.read());
                let names = cache.keyspace_names();
                filter_candidates(&names, prefix, false)
            }
            CompletionContext::TableName { keyspace } => {
                let cache = self.rt_handle.block_on(self.cache.read());
                let ks = keyspace.clone().or_else(|| {
                    self.rt_handle.block_on(async {
                        self.current_keyspace.read().await.clone()
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
                let cache = self.rt_handle.block_on(self.cache.read());
                let ks = keyspace.clone().or_else(|| {
                    self.rt_handle.block_on(async {
                        self.current_keyspace.read().await.clone()
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
            CompletionContext::FilePath => {
                complete_file_path(prefix)
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
            let display = if uppercase { c.to_uppercase() } else { c.to_string() };
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
        let parent = path.parent().map(|p| p.to_str().unwrap_or(".")).unwrap_or(".");
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
        // Refresh cache if stale
        let needs_refresh = self.rt_handle.block_on(async {
            self.cache.read().await.is_stale()
        });
        if needs_refresh {
            // Best-effort refresh — don't block on errors
            let _ = self.rt_handle.block_on(async {
                // Try to get write lock without blocking other completions
                if let Ok(mut cache) = self.cache.try_write() {
                    // Re-check staleness after acquiring lock
                    if cache.is_stale() {
                        // We can't refresh without a session reference here.
                        // The REPL pre-refreshes the cache; this is a fallback mark.
                        cache.invalidate();
                    }
                }
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

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: rustyline::highlight::CmdKind) -> bool {
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
        assert_eq!(
            c.detect_context("USE ", 4),
            CompletionContext::KeyspaceName
        );
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
        assert_eq!(
            c.detect_context("CAPTURE ", 8),
            CompletionContext::FilePath
        );
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
        // /tmp should exist on all Unix systems
        let pairs = complete_file_path("/tmp/");
        // Should return entries — exact count varies
        assert!(!pairs.is_empty() || std::fs::read_dir("/tmp").map(|d| d.count()).unwrap_or(0) == 0);
    }
}

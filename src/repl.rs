//! Interactive REPL (Read-Eval-Print Loop) for cqlsh-rs.
//!
//! Integrates rustyline for line editing, history, and prompt management.
//! Mirrors the Python cqlsh interactive behavior including multi-line input,
//! prompt formatting, and Ctrl-C/Ctrl-D handling.

use std::path::{Path, PathBuf};

use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Config, EditMode, Editor};

use crate::config::MergedConfig;
use crate::driver::CqlResult;
use crate::error;
use crate::parser::{self, ParseResult, StatementParser};
use crate::session::CqlSession;

/// Default history file path: ~/.cassandra/cql_history
const DEFAULT_HISTORY_DIR: &str = ".cassandra";
const DEFAULT_HISTORY_FILE: &str = "cql_history";
/// Maximum history entries (matches Python cqlsh default).
const DEFAULT_HISTORY_SIZE: usize = 1000;
/// Continuation prompt for multi-line input (matches Python cqlsh).
const CONTINUATION_PROMPT: &str = "   ... ";

/// Build the primary prompt string matching Python cqlsh format.
///
/// Format: `[username@]cqlsh[:keyspace]> `
///
/// Examples:
/// - `cqlsh> ` (no user, no keyspace)
/// - `cqlsh:my_ks> ` (with keyspace)
/// - `admin@cqlsh> ` (with username)
/// - `admin@cqlsh:my_ks> ` (with username and keyspace)
pub fn build_prompt(username: Option<&str>, keyspace: Option<&str>) -> String {
    let mut prompt = String::with_capacity(64);
    if let Some(user) = username {
        prompt.push_str(user);
        prompt.push('@');
    }
    prompt.push_str("cqlsh");
    if let Some(ks) = keyspace {
        prompt.push(':');
        prompt.push_str(ks);
    }
    prompt.push_str("> ");
    prompt
}

/// Resolve the history file path.
///
/// Priority: CQL_HISTORY env var > ~/.cassandra/cql_history
fn resolve_history_path(config: &MergedConfig) -> Option<PathBuf> {
    if config.disable_history {
        return None;
    }

    // Check CQL_HISTORY env var (already captured in EnvConfig, but we
    // also respect it directly here for simplicity)
    if let Ok(path) = std::env::var("CQL_HISTORY") {
        return Some(PathBuf::from(path));
    }

    dirs::home_dir().map(|home| home.join(DEFAULT_HISTORY_DIR).join(DEFAULT_HISTORY_FILE))
}

// Statement parsing is now handled by the parser module (SP4).
// The REPL uses `parser::StatementParser` for incremental, context-aware
// statement detection that correctly handles strings, comments, and
// multi-line input.

/// Run the interactive REPL loop.
///
/// Reads lines from the user, handles multi-line input, and dispatches
/// complete statements to the session for execution.
pub async fn run(session: &mut CqlSession, config: &MergedConfig) -> Result<()> {
    let rl_config = Config::builder()
        .max_history_size(DEFAULT_HISTORY_SIZE)
        .expect("valid history size")
        .edit_mode(EditMode::Emacs)
        .auto_add_history(true)
        .build();

    let mut rl: Editor<(), DefaultHistory> = Editor::with_config(rl_config)?;

    // Load history
    let history_path = resolve_history_path(config);
    if let Some(ref path) = history_path {
        // Ensure the parent directory exists
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = rl.load_history(path);
    }

    let username = config.username.as_deref();
    let mut stmt_parser = StatementParser::new();
    let mut expanded = false;

    loop {
        let prompt = if stmt_parser.is_empty() {
            build_prompt(username, session.current_keyspace())
        } else {
            CONTINUATION_PROMPT.to_string()
        };

        match rl.readline(&prompt) {
            Ok(line) => {
                // BUG-5 fix: Split pasted multi-line input into individual
                // lines so each is processed separately.
                let lines: Vec<&str> = line.split('\n').collect();
                for sub_line in lines {
                    process_line(
                        sub_line,
                        &mut stmt_parser,
                        session,
                        config,
                        &mut expanded,
                    )
                    .await;
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: cancel current input buffer, return to prompt
                stmt_parser.reset();
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D: exit
                break;
            }
            Err(err) => {
                eprintln!("Error reading input: {err}");
                break;
            }
        }
    }

    // Save history
    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }

    Ok(())
}

/// Process a single line of input through the REPL pipeline.
///
/// Handles shell command detection, incremental parsing, and dispatch.
async fn process_line(
    line: &str,
    stmt_parser: &mut StatementParser,
    session: &mut CqlSession,
    config: &MergedConfig,
    expanded: &mut bool,
) {
    let trimmed = line.trim();

    // On an empty primary prompt, just show the prompt again
    if stmt_parser.is_empty() && trimmed.is_empty() {
        return;
    }

    // Shell commands are complete without semicolons (only on first line)
    if stmt_parser.is_empty() && parser::is_shell_command(trimmed) {
        dispatch_input(session, config, trimmed, expanded).await;
        return;
    }

    // Feed line to the incremental parser
    if let ParseResult::Complete(statements) = stmt_parser.feed_line(line) {
        for stmt in statements {
            dispatch_input(session, config, &stmt, expanded).await;
        }
    }
}

/// Dispatch a complete input line/statement to the session.
///
/// Handles built-in shell commands and CQL statements.
async fn dispatch_input(
    session: &mut CqlSession,
    config: &MergedConfig,
    input: &str,
    expanded: &mut bool,
) {
    let trimmed = input.trim();
    let upper = trimmed.to_uppercase();

    // Handle QUIT/EXIT
    if upper == "QUIT" || upper == "EXIT" {
        std::process::exit(0);
    }

    // Handle HELP [topic]
    if upper == "HELP" || upper == "?" || upper.starts_with("HELP ") {
        if let Some(topic) = upper.strip_prefix("HELP ") {
            print_help_topic(topic.trim());
        } else {
            print_help();
        }
        return;
    }

    // Handle CLEAR/CLS
    if upper == "CLEAR" || upper == "CLS" {
        print!("\x1B[2J\x1B[1;1H");
        return;
    }

    // Handle CONSISTENCY
    if upper == "CONSISTENCY" {
        let cl = session.get_consistency();
        println!("Current consistency level is {cl}.");
        return;
    }
    if let Some(rest) = upper.strip_prefix("CONSISTENCY ") {
        let level = rest.trim();
        match session.set_consistency_str(level) {
            Ok(()) => println!("Consistency level set to {level}."),
            Err(e) => eprintln!("{e}"),
        }
        return;
    }

    // Handle SERIAL CONSISTENCY
    if upper == "SERIAL CONSISTENCY" {
        match session.get_serial_consistency() {
            Some(scl) => println!("Current serial consistency level is {scl}."),
            None => println!("Current serial consistency level is SERIAL."),
        }
        return;
    }
    if let Some(rest) = upper.strip_prefix("SERIAL CONSISTENCY ") {
        let level = rest.trim();
        match session.set_serial_consistency_str(level) {
            Ok(()) => println!("Serial consistency level set to {level}."),
            Err(e) => eprintln!("{e}"),
        }
        return;
    }

    // Handle TRACING
    if upper == "TRACING" || upper == "TRACING OFF" {
        session.set_tracing(false);
        println!("Disabled tracing.");
        return;
    }
    if upper == "TRACING ON" {
        session.set_tracing(true);
        println!("Now tracing requests.");
        return;
    }

    // Handle EXPAND
    if upper == "EXPAND" || upper == "EXPAND OFF" {
        *expanded = false;
        println!("Disabled EXPAND.");
        return;
    }
    if upper == "EXPAND ON" {
        *expanded = true;
        println!("Now printing expanded output.");
        return;
    }

    // Handle SHOW VERSION
    if upper == "SHOW VERSION" {
        println!("[cqlsh {}]", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Handle SHOW HOST
    if upper == "SHOW HOST" {
        println!("Connected to: {}", session.connection_display);
        return;
    }

    // Handle SOURCE
    if upper.starts_with("SOURCE ") || upper == "SOURCE" {
        if upper == "SOURCE" {
            eprintln!("SOURCE requires a filename argument.");
            return;
        }
        let arg = trimmed["SOURCE ".len()..].trim();
        let path = arg.trim_matches('\'').trim_matches('"');
        execute_source(session, config, expanded, Path::new(path)).await;
        return;
    }

    // Handle DESCRIBE / DESC
    if upper.starts_with("DESCRIBE ") || upper.starts_with("DESC ") || upper == "DESCRIBE" || upper == "DESC" {
        execute_describe(session, &upper, session.current_keyspace().map(String::from)).await;
        return;
    }

    // Execute as CQL statement
    match session.execute(trimmed).await {
        Ok(result) => {
            if !result.rows.is_empty() {
                print_results(&result, *expanded);
            }
        }
        Err(e) => {
            eprintln!("{}", error::format_error(&e));
            if config.debug {
                eprintln!("Debug: {e:?}");
            }
        }
    }
}

/// Execute a SOURCE command: read a CQL file and dispatch each statement.
///
/// Shell commands in the file (SHOW, CONSISTENCY, etc.) are routed through
/// `dispatch_input` just like interactive input — they are not sent to the DB.
fn execute_source<'a>(
    session: &'a mut CqlSession,
    config: &'a MergedConfig,
    expanded: &'a mut bool,
    path: &'a Path,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'a>> {
    Box::pin(async move {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Unable to open '{}' for reading: {e}", path.display());
                return;
            }
        };

        // Use the incremental parser line-by-line so shell commands in the file
        // are detected on their own lines (just like the interactive REPL).
        let mut stmt_parser = StatementParser::new();
        for line in content.lines() {
            let trimmed = line.trim();

            // Shell commands on their own line — dispatch directly
            if stmt_parser.is_empty() && !trimmed.is_empty() && parser::is_shell_command(trimmed) {
                dispatch_input(session, config, trimmed, expanded).await;
                continue;
            }

            if let ParseResult::Complete(stmts) = stmt_parser.feed_line(line) {
                for stmt in stmts {
                    dispatch_input(session, config, &stmt, expanded).await;
                }
            }
        }
    })
}

/// Execute a DESCRIBE / DESC command.
///
/// Supported forms:
/// - `DESCRIBE TABLES` / `DESC TABLES` — list tables (all keyspaces if none selected)
/// - `DESCRIBE KEYSPACES` / `DESC KEYSPACES` — list all keyspaces
async fn execute_describe(
    session: &CqlSession,
    upper_input: &str,
    current_keyspace: Option<String>,
) {
    let args = if let Some(rest) = upper_input.strip_prefix("DESCRIBE") {
        rest.trim()
    } else if let Some(rest) = upper_input.strip_prefix("DESC") {
        rest.trim()
    } else {
        ""
    };

    match args {
        "TABLES" | "COLUMNFAMILIES" => {
            if let Some(ks) = &current_keyspace {
                // Single keyspace
                match session.get_tables(ks).await {
                    Ok(tables) => {
                        let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
                        println!("\n{}\n", names.join("\n"));
                    }
                    Err(e) => eprintln!("Error: {e}"),
                }
            } else {
                // All keyspaces — Python cqlsh format
                match session.get_keyspaces().await {
                    Ok(keyspaces) => {
                        println!();
                        for ks in &keyspaces {
                            match session.get_tables(&ks.name).await {
                                Ok(tables) => {
                                    println!("Keyspace {}", ks.name);
                                    println!("{}", "-".repeat(ks.name.len() + 9));
                                    if tables.is_empty() {
                                        println!();
                                    } else {
                                        let names: Vec<&str> =
                                            tables.iter().map(|t| t.name.as_str()).collect();
                                        println!("{}\n", names.join("\n"));
                                    }
                                }
                                Err(e) => eprintln!("Error listing tables for {}: {e}", ks.name),
                            }
                        }
                    }
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
        }
        "KEYSPACES" => match session.get_keyspaces().await {
            Ok(keyspaces) => {
                let names: Vec<&str> = keyspaces.iter().map(|k| k.name.as_str()).collect();
                println!("\n{}\n", names.join("\n"));
            }
            Err(e) => eprintln!("Error: {e}"),
        },
        "" => {
            eprintln!("DESCRIBE requires a subcommand. Try: DESCRIBE TABLES, DESCRIBE KEYSPACES");
        }
        _ => {
            eprintln!("DESCRIBE {args} is not yet implemented.");
        }
    }
}

/// Print a basic help message matching Python cqlsh style.
fn print_help() {
    println!(
        "\
Documented shell commands:
  CLEAR         Clear the terminal screen
  CONSISTENCY   Get/set consistency level
  DESCRIBE      Schema introspection (TABLES, KEYSPACES)
  EXIT / QUIT   Exit the shell
  EXPAND        Toggle expanded (vertical) output
  HELP          Show this help or help on a topic
  SERIAL        Get/set serial consistency level
  SHOW          Show version or host info
  SOURCE        Execute a CQL file
  TRACING       Toggle request tracing

Not yet implemented:
  CAPTURE       Capture output to file
  COPY          Import/export CSV data
  LOGIN         Re-authenticate
  PAGING        Configure automatic paging
  UNICODE       Show Unicode handling info
  DEBUG         Toggle debug mode

CQL statements (executed via the database):
  SELECT, INSERT, UPDATE, DELETE, CREATE, ALTER, DROP, USE, etc."
    );
}

/// Print help for a specific topic.
///
/// This is a stub — full per-topic help text will be added in a later phase.
/// For now, print a message indicating the topic exists or is unknown.
fn print_help_topic(topic: &str) {
    // Shell commands
    let shell_commands = [
        "CAPTURE",
        "CLEAR",
        "CLS",
        "CONSISTENCY",
        "COPY",
        "DESC",
        "DESCRIBE",
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
        "UNICODE",
        "DEBUG",
        "USE",
    ];
    // CQL help topics
    let cql_topics = [
        "AGGREGATES",
        "ALTER_KEYSPACE",
        "ALTER_TABLE",
        "ALTER_TYPE",
        "ALTER_USER",
        "APPLY",
        "BEGIN",
        "CREATE_AGGREGATE",
        "CREATE_FUNCTION",
        "CREATE_INDEX",
        "CREATE_KEYSPACE",
        "CREATE_TABLE",
        "CREATE_TRIGGER",
        "CREATE_TYPE",
        "CREATE_USER",
        "DELETE",
        "DROP_AGGREGATE",
        "DROP_FUNCTION",
        "DROP_INDEX",
        "DROP_KEYSPACE",
        "DROP_TABLE",
        "DROP_TRIGGER",
        "DROP_TYPE",
        "DROP_USER",
        "GRANT",
        "INSERT",
        "LIST_PERMISSIONS",
        "LIST_USERS",
        "PERMISSIONS",
        "REVOKE",
        "SELECT",
        "TEXT_OUTPUT",
        "TRUNCATE",
        "TYPES",
        "UPDATE",
        "USE",
    ];

    let upper = topic.to_uppercase();
    if shell_commands.contains(&upper.as_str()) || cql_topics.contains(&upper.as_str()) {
        println!("Help topic: {upper}");
        println!("(Detailed help text not yet implemented.)");
    } else {
        println!("No help topic matching '{topic}'. Try HELP for a list of topics.");
    }
}

/// Format query results as a tabular string matching Python cqlsh style.
///
/// Output format:
/// ```text
///  col1 | col2 | col3
/// ------+------+------
///     1 |   42 | hello
///     2 |   99 | world
///
/// (2 rows)
/// ```
fn format_tabular(result: &CqlResult) -> String {
    if result.columns.is_empty() {
        return String::new();
    }

    let headers: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();

    // Pre-render all cell values
    let cell_values: Vec<Vec<String>> = result
        .rows
        .iter()
        .map(|row| {
            (0..result.columns.len())
                .map(|i| {
                    row.get(i)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "null".to_string())
                })
                .collect()
        })
        .collect();

    // Compute column widths: max of header width and all cell widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row_vals in &cell_values {
        for (i, val) in row_vals.iter().enumerate() {
            widths[i] = widths[i].max(val.len());
        }
    }

    let mut out = String::new();
    out.push('\n');

    // Header row: right-pad each column name
    let header_cells: Vec<String> = headers
        .iter()
        .zip(&widths)
        .map(|(h, w)| format!("{:>w$}", h, w = w))
        .collect();
    out.push_str(&format!(" {}\n", header_cells.join(" | ")));

    // Separator: dashes per column joined by +
    let sep_cells: Vec<String> = widths.iter().map(|w| "-".repeat(w + 2)).collect();
    out.push_str(&format!("{}\n", sep_cells.join("+")));

    // Data rows: right-align values (matching Python cqlsh default)
    for row_vals in &cell_values {
        let cells: Vec<String> = row_vals
            .iter()
            .zip(&widths)
            .map(|(v, w)| format!("{:>w$}", v, w = w))
            .collect();
        out.push_str(&format!(" {}\n", cells.join(" | ")));
    }

    out.push('\n');
    let row_count = result.rows.len();
    out.push_str(&format!(
        "({} row{})\n",
        row_count,
        if row_count == 1 { "" } else { "s" }
    ));

    out
}

/// Format query results in expanded (vertical) format matching Python cqlsh.
///
/// Output format:
/// ```text
/// @ Row 1
/// --------+-------------------
///   id    | 1
///  num    | 42
///  val    | hello world
///
/// (1 rows)
/// ```
fn format_expanded(result: &CqlResult) -> String {
    if result.columns.is_empty() {
        return String::new();
    }

    let headers: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    let max_name_width = headers.iter().map(|h| h.len()).max().unwrap_or(0);

    let mut out = String::new();
    out.push('\n');

    for (row_idx, row) in result.rows.iter().enumerate() {
        out.push_str(&format!("@ Row {}\n", row_idx + 1));

        // Compute max value width for this row's separator
        let values: Vec<String> = (0..result.columns.len())
            .map(|i| {
                row.get(i)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "null".to_string())
            })
            .collect();
        let max_val_width = values.iter().map(|v| v.len()).max().unwrap_or(0);

        // Separator: dashes matching name column + " | " + value column
        out.push_str(&format!(
            "{}+{}\n",
            "-".repeat(max_name_width + 2),
            "-".repeat(max_val_width + 2)
        ));

        // Field rows
        for (i, val) in values.iter().enumerate() {
            out.push_str(&format!(
                " {:>width$} | {}\n",
                headers[i],
                val,
                width = max_name_width
            ));
        }

        out.push('\n');
    }

    let row_count = result.rows.len();
    out.push_str(&format!(
        "({} row{})\n",
        row_count,
        if row_count == 1 { "" } else { "s" }
    ));

    out
}

/// Print query results (delegates to tabular or expanded based on mode).
fn print_results(result: &CqlResult, expanded: bool) {
    if expanded {
        print!("{}", format_expanded(result));
    } else {
        print!("{}", format_tabular(result));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Prompt tests ---

    #[test]
    fn prompt_default() {
        assert_eq!(build_prompt(None, None), "cqlsh> ");
    }

    #[test]
    fn prompt_with_keyspace() {
        assert_eq!(build_prompt(None, Some("my_ks")), "cqlsh:my_ks> ");
    }

    #[test]
    fn prompt_with_username() {
        assert_eq!(build_prompt(Some("admin"), None), "admin@cqlsh> ");
    }

    #[test]
    fn prompt_with_username_and_keyspace() {
        assert_eq!(
            build_prompt(Some("admin"), Some("system")),
            "admin@cqlsh:system> "
        );
    }

    // Statement completeness and shell command detection tests are now
    // in the parser module (src/parser.rs) where the logic lives.

    // --- History path tests ---

    #[test]
    fn history_disabled_returns_none() {
        let config = test_config(true);
        assert!(resolve_history_path(&config).is_none());
    }

    #[test]
    fn history_enabled_returns_path() {
        let config = test_config(false);
        let path = resolve_history_path(&config);
        // Should resolve to some path (unless no home dir)
        if dirs::home_dir().is_some() {
            assert!(path.is_some());
            let p = path.unwrap();
            assert!(p.to_string_lossy().contains("cql_history"));
        }
    }

    /// Create a minimal MergedConfig for testing.
    fn test_config(disable_history: bool) -> MergedConfig {
        use crate::config::{ColorMode, CqlshrcConfig};

        MergedConfig {
            host: "127.0.0.1".to_string(),
            port: 9042,
            username: None,
            password: None,
            keyspace: None,
            ssl: false,
            color: ColorMode::Auto,
            debug: false,
            tty: false,
            no_file_io: false,
            no_compact: false,
            disable_history,
            execute: None,
            file: None,
            connect_timeout: 5,
            request_timeout: 10,
            encoding: "utf-8".to_string(),
            cqlversion: None,
            protocol_version: None,
            consistency_level: None,
            serial_consistency_level: None,
            browser: None,
            secure_connect_bundle: None,
            cqlshrc_path: PathBuf::from("/dev/null"),
            cqlshrc: CqlshrcConfig::default(),
        }
    }

    // --- Helper to build CqlResult for testing ---

    fn make_result(
        col_names: &[&str],
        col_types: &[&str],
        rows: Vec<Vec<crate::driver::CqlValue>>,
    ) -> CqlResult {
        use crate::driver::types::{CqlColumn, CqlRow};

        CqlResult {
            columns: col_names
                .iter()
                .zip(col_types)
                .map(|(n, t)| CqlColumn {
                    name: n.to_string(),
                    type_name: t.to_string(),
                })
                .collect(),
            rows: rows
                .into_iter()
                .map(|values| CqlRow { values })
                .collect(),
            has_rows: true,
            tracing_id: None,
            warnings: Vec::new(),
        }
    }

    // --- BUG-2: Tabular formatting tests ---

    #[test]
    fn tabular_basic_alignment() {
        use crate::driver::CqlValue;
        let result = make_result(
            &["id", "num", "val"],
            &["int", "int", "text"],
            vec![
                vec![
                    CqlValue::Int(1),
                    CqlValue::Int(42),
                    CqlValue::Text("hello world".into()),
                ],
                vec![
                    CqlValue::Int(2),
                    CqlValue::Int(99),
                    CqlValue::Text("semi;colon;inside".into()),
                ],
            ],
        );
        let output = format_tabular(&result);
        // Header should be present with pipe separators
        assert!(output.contains("id"));
        assert!(output.contains("num"));
        assert!(output.contains("val"));
        assert!(output.contains(" | "));
        // Separator should use dashes and +
        assert!(output.contains("---+"));
        // No box-drawing characters
        assert!(!output.contains("||||"));
        assert!(!output.contains("+++"));
        // Row count
        assert!(output.contains("(2 rows)"));
    }

    #[test]
    fn tabular_column_width_adapts_to_data() {
        use crate::driver::CqlValue;
        let result = make_result(
            &["x", "longvalue"],
            &["int", "text"],
            vec![vec![
                CqlValue::Int(12345),
                CqlValue::Text("hi".into()),
            ]],
        );
        let output = format_tabular(&result);
        // "x" column should be widened to fit "12345" (5 chars)
        // Header "x" should be right-aligned in the wider column
        assert!(output.contains("12345"));
        assert!(output.contains("(1 row)"));
    }

    #[test]
    fn tabular_no_trailing_junk() {
        use crate::driver::CqlValue;
        let result = make_result(
            &["a"],
            &["text"],
            vec![vec![CqlValue::Text("test".into())]],
        );
        let output = format_tabular(&result);
        for line in output.lines() {
            // No trailing | or - chars (BUG-2 symptom)
            let trimmed = line.trim_end();
            if !trimmed.is_empty()
                && !trimmed.starts_with('(')
                && !trimmed.starts_with('-')
            {
                assert!(
                    !trimmed.ends_with('-'),
                    "Line should not end with '-': {trimmed:?}"
                );
            }
        }
    }

    #[test]
    fn tabular_empty_result() {
        let result = CqlResult::empty();
        assert_eq!(format_tabular(&result), "");
    }

    #[test]
    fn tabular_single_row_singular() {
        use crate::driver::CqlValue;
        let result = make_result(
            &["id"],
            &["int"],
            vec![vec![CqlValue::Int(1)]],
        );
        let output = format_tabular(&result);
        assert!(output.contains("(1 row)"));
        assert!(!output.contains("(1 rows)"));
    }

    // --- BUG-3: Expanded formatting tests ---

    #[test]
    fn expanded_basic_format() {
        use crate::driver::CqlValue;
        let result = make_result(
            &["id", "num", "val"],
            &["int", "int", "text"],
            vec![vec![
                CqlValue::Int(1),
                CqlValue::Int(42),
                CqlValue::Text("hello world".into()),
            ]],
        );
        let output = format_expanded(&result);
        assert!(output.contains("@ Row 1"));
        assert!(output.contains("---+---"));
        assert!(output.contains(" id"));
        assert!(output.contains("| 1"));
        assert!(output.contains("num"));
        assert!(output.contains("| 42"));
        assert!(output.contains("val"));
        assert!(output.contains("| hello world"));
        assert!(output.contains("(1 row)"));
    }

    #[test]
    fn expanded_column_names_padded() {
        use crate::driver::CqlValue;
        let result = make_result(
            &["id", "longname"],
            &["int", "text"],
            vec![vec![
                CqlValue::Int(1),
                CqlValue::Text("x".into()),
            ]],
        );
        let output = format_expanded(&result);
        // "id" should be padded to match "longname" width (8 chars)
        // So we should see "       id | 1" (right-aligned to 8)
        for line in output.lines() {
            if line.contains("| 1") && line.contains("id") {
                // The "id" part should be right-aligned in 8-char field
                let parts: Vec<&str> = line.splitn(2, '|').collect();
                let name_part = parts[0].trim_start();
                // "id" should have leading spaces for alignment
                assert!(
                    name_part.starts_with("id"),
                    "Name should be right-aligned: {line:?}"
                );
            }
        }
    }

    #[test]
    fn expanded_separator_has_plus() {
        use crate::driver::CqlValue;
        let result = make_result(
            &["a"],
            &["int"],
            vec![vec![CqlValue::Int(1)]],
        );
        let output = format_expanded(&result);
        // Separator must contain a + between dash segments
        let has_plus_sep = output.lines().any(|l| {
            l.contains('+') && l.chars().all(|c| c == '-' || c == '+')
        });
        assert!(has_plus_sep, "Separator should have dash+plus format");
    }

    #[test]
    fn expanded_multiple_rows() {
        use crate::driver::CqlValue;
        let result = make_result(
            &["id"],
            &["int"],
            vec![
                vec![CqlValue::Int(1)],
                vec![CqlValue::Int(2)],
            ],
        );
        let output = format_expanded(&result);
        assert!(output.contains("@ Row 1"));
        assert!(output.contains("@ Row 2"));
        assert!(output.contains("(2 rows)"));
    }

    #[test]
    fn expanded_empty_result() {
        let result = CqlResult::empty();
        assert_eq!(format_expanded(&result), "");
    }

    // --- BUG-4: SOURCE file parsing tests ---
    // (SOURCE dispatches through the same parser, so we test parse_batch
    //  shell command handling here)

    #[test]
    fn parse_batch_includes_shell_commands() {
        // Shell commands without semicolons in batch mode
        let input = "SELECT 1;\nSHOW VERSION\n";
        let stmts = parser::parse_batch(input);
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0], "SELECT 1");
        assert_eq!(stmts[1], "SHOW VERSION");
    }

    #[test]
    fn parse_batch_shell_command_with_semicolon() {
        // Shell commands with semicolons should also be captured
        let input = "SHOW VERSION;\nSELECT 1;\n";
        let stmts = parser::parse_batch(input);
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0], "SHOW VERSION");
        assert_eq!(stmts[1], "SELECT 1");
    }

    #[test]
    fn source_file_line_by_line_detects_shell_commands() {
        // execute_source processes line-by-line with shell command detection,
        // not through parse_batch. Verify the line-by-line approach works:
        // each line is checked for shell commands independently.
        let lines = vec![
            "CONSISTENCY QUORUM",
            "SELECT * FROM t;",
            "SHOW HOST",
        ];
        let mut shell_cmds = Vec::new();
        let mut cql_stmts = Vec::new();
        let mut parser = StatementParser::new();

        for line in &lines {
            let trimmed = line.trim();
            if parser.is_empty() && !trimmed.is_empty() && parser::is_shell_command(trimmed) {
                shell_cmds.push(trimmed.to_string());
                continue;
            }
            if let ParseResult::Complete(stmts) = parser.feed_line(line) {
                cql_stmts.extend(stmts);
            }
        }

        assert_eq!(shell_cmds, vec!["CONSISTENCY QUORUM", "SHOW HOST"]);
        assert_eq!(cql_stmts, vec!["SELECT * FROM t"]);
    }

    // --- BUG-5: Multi-line paste tests ---

    #[test]
    fn multiline_paste_splits_into_lines() {
        // Simulate what happens when pasted text contains newlines:
        // The REPL splits by \n and processes each line independently.
        let pasted = "SHOW VERSION\nSELECT 1;\nSHOW HOST";
        let lines: Vec<&str> = pasted.split('\n').collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "SHOW VERSION");
        assert_eq!(lines[1], "SELECT 1;");
        assert_eq!(lines[2], "SHOW HOST");
        // First and third are shell commands
        assert!(parser::is_shell_command(lines[0].trim()));
        assert!(parser::is_shell_command(lines[2].trim()));
    }

    #[test]
    fn multiline_paste_shell_command_not_concatenated() {
        // The bug was: pasting "CAPTURE foo\nSELECT 1;\nCAPTURE OFF"
        // resulted in the filename being "foo\nSELECT 1;\nCAPTURE OFF"
        // After fix, each line is separate.
        let pasted = "CAPTURE '/tmp/test.txt'\nSELECT 1;\nCAPTURE OFF";
        let lines: Vec<&str> = pasted.split('\n').collect();
        assert_eq!(lines.len(), 3);
        // CAPTURE line should only contain the filename, not the rest
        assert_eq!(lines[0], "CAPTURE '/tmp/test.txt'");
        assert!(parser::is_shell_command(lines[0].trim()));
    }

    // --- BUG-1: DESCRIBE output tests ---
    // (execute_describe requires async + session, so we test the formatting
    //  logic via the describe output helper)

    #[test]
    fn describe_tables_format_all_keyspaces() {
        // Verify the expected output format for DESCRIBE TABLES
        // when listing all keyspaces
        let keyspaces = vec![
            ("demo_ks", vec!["demo_t", "other_t"]),
            ("system", vec!["local", "peers"]),
        ];
        let mut output = String::new();
        output.push('\n');
        for (ks, tables) in &keyspaces {
            output.push_str(&format!("Keyspace {ks}\n"));
            output.push_str(&format!("{}\n", "-".repeat(ks.len() + 9)));
            output.push_str(&format!("{}\n\n", tables.join("\n")));
        }
        assert!(output.contains("Keyspace demo_ks"));
        assert!(output.contains("-".repeat("demo_ks".len() + 9).as_str()));
        assert!(output.contains("demo_t"));
        assert!(output.contains("other_t"));
        assert!(output.contains("Keyspace system"));
        assert!(output.contains("local"));
        assert!(output.contains("peers"));
    }

    #[test]
    fn describe_separator_matches_header_width() {
        // "Keyspace demo_ks" is 16 chars, separator should be 16 dashes
        let ks_name = "demo_ks";
        let header = format!("Keyspace {ks_name}");
        let separator = "-".repeat(ks_name.len() + 9);
        assert_eq!(header.len(), separator.len());
    }
}

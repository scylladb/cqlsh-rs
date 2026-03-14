//! Interactive REPL (Read-Eval-Print Loop) for cqlsh-rs.
//!
//! Integrates rustyline for line editing, history, and prompt management.
//! Mirrors the Python cqlsh interactive behavior including multi-line input,
//! prompt formatting, and Ctrl-C/Ctrl-D handling.

use std::path::PathBuf;

use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Config, EditMode, Editor};

use crate::config::MergedConfig;
use crate::driver::CqlResult;
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

    loop {
        let prompt = if stmt_parser.is_empty() {
            build_prompt(username, session.current_keyspace())
        } else {
            CONTINUATION_PROMPT.to_string()
        };

        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();

                // On an empty primary prompt, just show the prompt again
                if stmt_parser.is_empty() && trimmed.is_empty() {
                    continue;
                }

                // Shell commands are complete without semicolons (only on first line)
                if stmt_parser.is_empty() && parser::is_shell_command(trimmed) {
                    dispatch_input(session, config, trimmed).await;
                    continue;
                }

                // Feed line to the incremental parser
                match stmt_parser.feed_line(&line) {
                    ParseResult::Complete(statements) => {
                        for stmt in statements {
                            dispatch_input(session, config, &stmt).await;
                        }
                    }
                    ParseResult::Incomplete => {
                        // Continue accumulating multi-line input
                    }
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

/// Dispatch a complete input line/statement to the session.
///
/// Handles built-in shell commands and CQL statements.
async fn dispatch_input(session: &mut CqlSession, config: &MergedConfig, input: &str) {
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

    // Execute as CQL statement
    match session.execute(trimmed).await {
        Ok(result) => {
            if !result.rows.is_empty() {
                print_basic_results(&result);
            }
        }
        Err(e) => {
            // Walk the error chain to find the most specific/useful message.
            // The outermost error is often a generic wrapper like "executing CQL query".
            let root = e.root_cause();
            eprintln!("ServerError: {root}");
            if config.debug {
                eprintln!("Debug: {e:?}");
            }
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
  EXIT / QUIT   Exit the shell
  HELP          Show this help or help on a topic
  SERIAL        Get/set serial consistency level
  SHOW          Show version or host info
  TRACING       Toggle request tracing

Not yet implemented:
  CAPTURE       Capture output to file
  COPY          Import/export CSV data
  DESCRIBE      Schema introspection
  EXPAND        Toggle expanded output
  LOGIN         Re-authenticate
  PAGING        Configure automatic paging
  SOURCE        Execute CQL file
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

/// Print query results in a basic tabular format.
///
/// This is a minimal implementation. SP6 (Output Formatting) will provide
/// the full comfy-table based formatter with proper alignment and colors.
fn print_basic_results(result: &CqlResult) {
    if result.columns.is_empty() {
        return;
    }

    // Print header
    let header: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    println!();
    println!(" {} ", header.join(" | "));
    println!(
        "{}",
        header
            .iter()
            .map(|h| "-".repeat(h.len() + 2))
            .collect::<Vec<_>>()
            .join("+")
    );

    // Print rows
    for row in &result.rows {
        let values: Vec<String> = (0..result.columns.len())
            .map(|i| {
                row.get(i)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "null".to_string())
            })
            .collect();
        println!(" {} ", values.join(" | "));
    }

    println!();
    let row_count = result.rows.len();
    println!(
        "({} row{})",
        row_count,
        if row_count == 1 { "" } else { "s" }
    );
    println!();
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
}

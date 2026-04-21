//! Interactive REPL (Read-Eval-Print Loop) for cqlsh-rs.
//!
//! Integrates rustyline for line editing, history, and prompt management.
//! Mirrors the Python cqlsh interactive behavior including multi-line input,
//! prompt formatting, and Ctrl-C/Ctrl-D handling.

use std::fs::File;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{CompletionType, Config, EditMode, Editor};
use tokio::sync::RwLock;

use crate::colorizer::CqlColorizer;
use crate::completer::CqlCompleter;
use crate::config::MergedConfig;
use crate::describe;
use crate::error;
use crate::formatter;
use crate::parser::{self, ParseResult, StatementParser};
use crate::schema_cache::SchemaCache;
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

/// Mutable shell state for commands like EXPAND, PAGING, and CAPTURE.
struct ShellState {
    /// Whether expanded (vertical) output is enabled.
    expand: bool,
    /// Whether to pipe output through the built-in pager.
    paging_enabled: bool,
    /// Whether stdout is a TTY (controls pager auto-disable).
    is_tty: bool,
    /// Whether debug mode is enabled (toggled via DEBUG command).
    debug: bool,
    /// Active CAPTURE file handle (output is tee'd to this file).
    capture_file: Option<File>,
    /// Path of the active capture file (for display).
    capture_path: Option<PathBuf>,
    /// Shared schema cache for tab completion (invalidated on DDL).
    schema_cache: Option<Arc<RwLock<SchemaCache>>>,
    /// Shared current keyspace for tab completion.
    shared_keyspace: Option<Arc<RwLock<Option<String>>>>,
    /// Output colorizer for result values, headers, and errors.
    colorizer: CqlColorizer,
}

impl ShellState {
    /// Write output line to both stdout and the capture file (if active).
    /// Used for short shell command output that doesn't need paging.
    fn outputln(&mut self, text: &str) {
        println!("{text}");
        if let Some(ref mut f) = self.capture_file {
            let _ = writeln!(f, "{text}");
        }
    }

    /// Display output, routing through the pager if enabled, and writing to capture file.
    /// An optional `title` is shown at the top of the pager (e.g., column names).
    fn display_output(&mut self, content: &[u8], title: &str) {
        // Write to capture file if active
        if let Some(ref mut f) = self.capture_file {
            let _ = f.write_all(content);
        }

        let text = String::from_utf8_lossy(content);

        // Route through pager or print directly
        if self.paging_enabled && self.is_tty {
            if crate::pager::page_content(&text, title).is_err() {
                // Fallback: print directly if pager fails
                print!("{text}");
            }
        } else {
            print!("{text}");
        }
    }
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
        .completion_type(CompletionType::List)
        .build();

    // Set up schema cache and tab completer
    let schema_cache = Arc::new(RwLock::new(SchemaCache::new()));
    let current_keyspace: Arc<RwLock<Option<String>>> =
        Arc::new(RwLock::new(session.current_keyspace().map(String::from)));

    // Initial schema cache population (best-effort)
    {
        let mut cache = schema_cache.write().await;
        if let Err(e) = cache.refresh(session).await {
            eprintln!("Warning: could not load schema for tab completion: {e}");
        }
    }

    // Resolve color mode: Auto → check if stdout is a terminal
    // --tty flag forces TTY behavior even when piped
    let is_tty = config.tty || std::io::stdout().is_terminal();
    let color_enabled = match config.color {
        crate::config::ColorMode::On => true,
        crate::config::ColorMode::Off => false,
        crate::config::ColorMode::Auto => is_tty,
    };

    let completer = CqlCompleter::new(
        Arc::clone(&schema_cache),
        Arc::clone(&current_keyspace),
        tokio::runtime::Handle::current(),
        color_enabled,
    );

    let mut rl: Editor<CqlCompleter, DefaultHistory> = Editor::with_config(rl_config)?;
    rl.set_helper(Some(completer));

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
    let colorizer = CqlColorizer::new(color_enabled);
    let mut shell = ShellState {
        expand: false,
        paging_enabled: true,
        is_tty,
        debug: config.debug,
        capture_file: None,
        capture_path: None,
        schema_cache: Some(Arc::clone(&schema_cache)),
        shared_keyspace: Some(Arc::clone(&current_keyspace)),
        colorizer,
    };

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
                    process_line(sub_line, &mut stmt_parser, session, config, &mut shell).await;
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
    shell: &mut ShellState,
) {
    let trimmed = line.trim();

    // On an empty primary prompt, just show the prompt again
    if stmt_parser.is_empty() && trimmed.is_empty() {
        return;
    }

    // Shell commands are complete without semicolons (only on first line)
    if stmt_parser.is_empty() && parser::is_shell_command(trimmed) {
        // Strip trailing semicolon before dispatch — is_shell_command tolerates
        // the semicolon for detection, but handlers expect clean input.
        let clean = trimmed.strip_suffix(';').unwrap_or(trimmed).trim_end();
        dispatch_input(session, config, shell, clean).await;
        return;
    }

    // Feed line to the incremental parser
    if let ParseResult::Complete(statements) = stmt_parser.feed_line(line) {
        for stmt in statements {
            dispatch_input(session, config, shell, &stmt).await;
        }
    }
}

/// Dispatch a complete input line/statement to the session.
///
/// Handles built-in shell commands and CQL statements.
/// Uses `Box::pin` to support recursive calls from `execute_source`.
fn dispatch_input<'a>(
    session: &'a mut CqlSession,
    config: &'a MergedConfig,
    shell: &'a mut ShellState,
    input: &'a str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'a>> {
    Box::pin(async move {
        let trimmed = input.trim();
        let upper = trimmed.to_uppercase();

        // Handle QUIT/EXIT
        if upper == "QUIT" || upper == "EXIT" {
            std::process::exit(0);
        }

        // Handle HELP [topic]
        if upper == "HELP" || upper == "?" || upper.starts_with("HELP ") {
            if let Some(topic) = upper.strip_prefix("HELP ") {
                print_help_topic(topic.trim(), &mut std::io::stdout());
            } else {
                print_help(&mut std::io::stdout());
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
            shell.outputln(&format!("Current consistency level is {cl}."));
            return;
        }
        if let Some(rest) = upper.strip_prefix("CONSISTENCY ") {
            let level = rest.trim();
            match session.set_consistency_str(level) {
                Ok(()) => shell.outputln(&format!("Consistency level set to {level}.")),
                Err(e) => eprintln!("{e}"),
            }
            return;
        }

        // Handle SERIAL CONSISTENCY
        if upper == "SERIAL CONSISTENCY" {
            match session.get_serial_consistency() {
                Some(scl) => shell.outputln(&format!("Current serial consistency level is {scl}.")),
                None => shell.outputln("Current serial consistency level is SERIAL."),
            }
            return;
        }
        if let Some(rest) = upper.strip_prefix("SERIAL CONSISTENCY ") {
            let level = rest.trim();
            match session.set_serial_consistency_str(level) {
                Ok(()) => shell.outputln(&format!("Serial consistency level set to {level}.")),
                Err(e) => eprintln!("{e}"),
            }
            return;
        }

        // Handle TRACING
        if upper == "TRACING" || upper == "TRACING OFF" {
            session.set_tracing(false);
            shell.outputln("Disabled tracing.");
            return;
        }
        if upper == "TRACING ON" {
            session.set_tracing(true);
            shell.outputln("Now tracing requests.");
            return;
        }

        // Handle EXPAND
        if upper == "EXPAND" {
            if shell.expand {
                shell.outputln("Expanded output is currently enabled. Use EXPAND OFF to disable.");
            } else {
                shell.outputln("Expanded output is currently disabled. Use EXPAND ON to enable.");
            }
            return;
        }
        if upper == "EXPAND ON" {
            shell.expand = true;
            shell.outputln("Now printing expanded output.");
            return;
        }
        if upper == "EXPAND OFF" {
            shell.expand = false;
            shell.outputln("Disabled expanded output.");
            return;
        }

        // Handle PAGING
        if upper == "PAGING" {
            if shell.paging_enabled {
                shell.outputln("Query paging is currently enabled. Use PAGING OFF to disable.");
            } else {
                shell.outputln("Query paging is currently disabled. Use PAGING ON to enable.");
            }
            return;
        }
        if upper == "PAGING ON" {
            shell.paging_enabled = true;
            shell.outputln("Now query paging is enabled.");
            return;
        }
        if upper == "PAGING OFF" {
            shell.paging_enabled = false;
            shell.outputln("Disabled paging.");
            return;
        }
        if upper.strip_prefix("PAGING ").is_some() {
            // Accept PAGING <N> for compatibility — enables paging
            shell.paging_enabled = true;
            shell.outputln("Now query paging is enabled.");
            return;
        }

        // Handle SOURCE
        if upper.starts_with("SOURCE ") {
            let path = trimmed["SOURCE ".len()..].trim();
            let path = strip_quotes(path);
            if config.no_file_io {
                eprintln!("File I/O is disabled (--no-file-io).");
            } else {
                execute_source(session, config, shell, path).await;
            }
            return;
        }
        if upper == "SOURCE" {
            eprintln!("SOURCE requires a file path argument.");
            return;
        }

        // Handle CAPTURE
        if upper == "CAPTURE" {
            match &shell.capture_path {
                Some(path) => {
                    shell.outputln(&format!("Currently capturing to '{}'.", path.display()))
                }
                None => shell.outputln("Not currently capturing."),
            }
            return;
        }
        if upper == "CAPTURE OFF" {
            if shell.capture_file.is_some() {
                let path = shell.capture_path.take().unwrap();
                shell.capture_file = None;
                shell.outputln(&format!(
                    "Stopped capture. Output saved to '{}'.",
                    path.display()
                ));
            } else {
                shell.outputln("Not currently capturing.");
            }
            return;
        }
        if upper.strip_prefix("CAPTURE ").is_some() {
            let path = trimmed["CAPTURE ".len()..].trim();
            let path = strip_quotes(path);
            if config.no_file_io {
                eprintln!("File I/O is disabled (--no-file-io).");
            } else {
                let expanded = expand_tilde(path);
                match File::create(&expanded) {
                    Ok(file) => {
                        shell.outputln(&format!(
                            "Now capturing query output to '{}'.",
                            expanded.display()
                        ));
                        shell.capture_file = Some(file);
                        shell.capture_path = Some(expanded);
                    }
                    Err(e) => eprintln!("Unable to open '{}' for writing: {e}", expanded.display()),
                }
            }
            return;
        }

        // Handle DEBUG
        if upper == "DEBUG" {
            if shell.debug {
                shell.outputln("Debug output is currently enabled. Use DEBUG OFF to disable.");
            } else {
                shell.outputln("Debug output is currently disabled. Use DEBUG ON to enable.");
            }
            return;
        }
        if upper == "DEBUG ON" {
            shell.debug = true;
            shell.outputln("Now printing debug output.");
            return;
        }
        if upper == "DEBUG OFF" {
            shell.debug = false;
            shell.outputln("Disabled debug output.");
            return;
        }

        // Handle UNICODE
        if upper == "UNICODE" {
            shell.outputln(&format!(
                "Encoding: {}\nDefault encoding: utf-8",
                config.encoding
            ));
            return;
        }

        // Handle LOGIN
        if upper == "LOGIN" {
            eprintln!("Usage: LOGIN <username> [<password>]");
            return;
        }
        if upper.starts_with("LOGIN ") {
            let args = trimmed["LOGIN ".len()..].trim();
            let parts: Vec<&str> = args.splitn(2, char::is_whitespace).collect();
            let new_user = parts[0].to_string();
            let new_pass = if parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                // Prompt for password
                eprint!("Password: ");
                let _ = io::stderr().flush();
                let mut pass = String::new();
                if io::stdin().read_line(&mut pass).is_ok() {
                    Some(pass.trim().to_string())
                } else {
                    None
                }
            };
            // Reconnect with new credentials
            let mut new_config = config.clone();
            new_config.username = Some(new_user);
            new_config.password = new_pass;
            match crate::session::CqlSession::connect(&new_config).await {
                Ok(new_session) => {
                    *session = new_session;
                    shell.outputln("Login successful.");
                }
                Err(e) => {
                    eprintln!("Login failed: {e}");
                }
            }
            return;
        }

        // Handle COPY TO
        if upper.starts_with("COPY ") && upper.contains(" TO ") {
            if config.no_file_io {
                eprintln!("File I/O is disabled (--no-file-io).");
            } else {
                match crate::copy::parse_copy_to(trimmed) {
                    Ok(cmd) => {
                        let ks = session.current_keyspace();
                        match crate::copy::execute_copy_to(session, &cmd, ks).await {
                            Ok(()) => {}
                            Err(e) => eprintln!("COPY TO error: {e}"),
                        }
                    }
                    Err(e) => eprintln!("Invalid COPY TO syntax: {e}"),
                }
            }
            return;
        }

        // Handle COPY FROM
        if upper.starts_with("COPY ") && upper.contains(" FROM ") {
            if config.no_file_io {
                eprintln!("File I/O is disabled (--no-file-io).");
            } else {
                match crate::copy::parse_copy_from(trimmed) {
                    Ok(cmd) => {
                        let ks = session.current_keyspace();
                        match crate::copy::execute_copy_from(session, &cmd, ks).await {
                            Ok(()) => {}
                            Err(e) => eprintln!("COPY FROM error: {e}"),
                        }
                    }
                    Err(e) => eprintln!("Invalid COPY FROM syntax: {e}"),
                }
            }
            return;
        }

        // Handle DESCRIBE / DESC
        if upper == "DESCRIBE"
            || upper == "DESC"
            || upper.starts_with("DESCRIBE ")
            || upper.starts_with("DESC ")
        {
            let args = if upper.starts_with("DESCRIBE ") {
                trimmed["DESCRIBE ".len()..].trim()
            } else if upper.starts_with("DESC ") {
                trimmed["DESC ".len()..].trim()
            } else {
                ""
            };
            let mut buf = Vec::new();
            match describe::execute(session, args, &mut buf).await {
                Ok(()) => shell.display_output(&buf, ""),
                Err(e) => eprintln!("Error: {e}"),
            }
            return;
        }

        // Handle SHOW VERSION
        if upper == "SHOW VERSION" {
            shell.outputln(&format!("[cqlsh {}]", env!("CARGO_PKG_VERSION")));
            return;
        }

        // Handle SHOW HOST
        if upper == "SHOW HOST" {
            shell.outputln(&format!("Connected to: {}", session.connection_display));
            return;
        }

        // Handle SHOW SESSION <uuid>
        if let Some(rest) = upper.strip_prefix("SHOW SESSION ") {
            let uuid_str = rest.trim();
            match uuid::Uuid::parse_str(uuid_str) {
                Ok(trace_id) => match session.get_trace_session(trace_id).await {
                    Ok(Some(trace)) => {
                        let mut buf = Vec::new();
                        formatter::print_trace(&trace, &shell.colorizer, &mut buf);
                        shell.display_output(&buf, "");
                    }
                    Ok(None) => eprintln!("Trace session {trace_id} not found."),
                    Err(e) => eprintln!("Error fetching trace: {e}"),
                },
                Err(_) => eprintln!("Invalid UUID: {uuid_str}"),
            }
            return;
        }
        if upper == "SHOW SESSION" {
            eprintln!("Usage: SHOW SESSION <trace-uuid>");
            return;
        }

        // Execute as CQL statement
        match session.execute(trimmed).await {
            Ok(result) => {
                // Sync current keyspace for tab completion after USE
                let upper_stmt = trimmed.to_uppercase();
                if upper_stmt.starts_with("USE ") {
                    if let Some(ref shared_ks) = shell.shared_keyspace {
                        let ks = session.current_keyspace().map(String::from);
                        let shared = Arc::clone(shared_ks);
                        *shared.write().await = ks;
                    }
                }

                // Invalidate schema cache after DDL statements
                if upper_stmt.starts_with("CREATE ")
                    || upper_stmt.starts_with("ALTER ")
                    || upper_stmt.starts_with("DROP ")
                {
                    if let Some(ref cache) = shell.schema_cache {
                        let mut c = cache.write().await;
                        c.invalidate();
                        let _ = c.refresh(session).await;
                    }
                }

                // Print warnings if present (red bold when colored)
                for warning in &result.warnings {
                    let msg = format!("Warnings: {warning}");
                    eprintln!("{}", shell.colorizer.colorize_warning(&msg));
                }

                if !result.columns.is_empty() {
                    // Build column list for pager title (sticky header context)
                    let col_title = result
                        .columns
                        .iter()
                        .map(|c| c.name.as_str())
                        .collect::<Vec<_>>()
                        .join(" | ");

                    let mut buf = Vec::new();
                    if shell.expand {
                        formatter::print_expanded(&result, &shell.colorizer, &mut buf);
                    } else {
                        formatter::print_tabular(&result, &shell.colorizer, &mut buf);
                    }
                    shell.display_output(&buf, &col_title);
                }

                // Print trace info if tracing is enabled
                if session.is_tracing_enabled() && !upper_stmt.contains("SYSTEM_TRACES") {
                    if let Some(trace_id) = result.tracing_id {
                        // Brief delay to allow trace data to propagate
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        match session.get_trace_session(trace_id).await {
                            Ok(Some(trace)) => {
                                let mut buf = Vec::new();
                                formatter::print_trace(&trace, &shell.colorizer, &mut buf);
                                shell.display_output(&buf, "");
                            }
                            Ok(None) => {
                                shell.outputln(&format!(
                                "Trace {trace_id} not yet available. Use SHOW SESSION {trace_id} to view later."
                            ));
                            }
                            Err(e) => {
                                eprintln!("Error fetching trace: {e}");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", error::format_error_colored(&e, &shell.colorizer));
                if config.debug {
                    eprintln!("Debug: {e:?}");
                }
            }
        }
    })
}

/// Print a basic help message matching Python cqlsh style.
pub fn print_help(writer: &mut dyn std::io::Write) {
    writeln!(
        writer,
        "\
Documented shell commands:
  CAPTURE       Capture output to file
  CLEAR         Clear the terminal screen
  CONSISTENCY   Get/set consistency level
  DEBUG         Toggle debug mode
  DESCRIBE      Schema introspection (CLUSTER, KEYSPACES, TABLE, etc.)
  EXIT / QUIT   Exit the shell
  EXPAND        Toggle expanded (vertical) output
  HELP          Show this help or help on a topic
  LOGIN         Re-authenticate with new credentials
  PAGING        Configure automatic paging
  SERIAL        Get/set serial consistency level
  SHOW          Show version, host, or session trace info
  SOURCE        Execute CQL from a file
  TRACING       Toggle request tracing
  UNICODE       Show Unicode character handling info

Partially implemented:
  COPY TO       Export table data to CSV file
  COPY FROM     Import CSV data into a table

CQL statements (executed via the database):
  SELECT, INSERT, UPDATE, DELETE, CREATE, ALTER, DROP, USE, etc."
    )
    .ok();
}

/// Print help for a specific topic.
///
/// This is a stub — full per-topic help text will be added in a later phase.
/// For now, print a message indicating the topic exists or is unknown.
pub fn print_help_topic(topic: &str, writer: &mut dyn std::io::Write) {
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
        writeln!(writer, "Help topic: {upper}").ok();
        writeln!(writer, "(Detailed help text not yet implemented.)").ok();
    } else {
        writeln!(
            writer,
            "No help topic matching '{topic}'. Try HELP for a list of topics."
        )
        .ok();
    }
}

/// Strip surrounding single or double quotes from a string.
fn strip_quotes(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Expand `~` at the start of a path to the user's home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

/// Execute a SOURCE file: read CQL statements and execute them sequentially.
///
/// Shell commands in the file (SHOW, CONSISTENCY, etc.) are routed through
/// `dispatch_input` just like interactive input — they are not sent to the DB.
fn execute_source<'a>(
    session: &'a mut CqlSession,
    config: &'a MergedConfig,
    shell: &'a mut ShellState,
    path: &'a str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'a>> {
    Box::pin(async move {
        let expanded = expand_tilde(path);
        let file = match File::open(&expanded) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Could not open '{}': {e}", expanded.display());
                return;
            }
        };

        let reader = io::BufReader::new(file);
        let mut parser = StatementParser::new();

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error reading '{}': {e}", expanded.display());
                    return;
                }
            };

            // Check if it's a shell command on a fresh line
            let trimmed = line.trim();
            if parser.is_empty() && !trimmed.is_empty() && parser::is_shell_command(trimmed) {
                dispatch_input(session, config, shell, trimmed).await;
                continue;
            }

            match parser.feed_line(&line) {
                ParseResult::Complete(statements) => {
                    for stmt in statements {
                        dispatch_input(session, config, shell, &stmt).await;
                    }
                }
                ParseResult::Incomplete => {}
            }
        }
    })
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

    // --- Helper function tests ---

    #[test]
    fn strip_quotes_double() {
        assert_eq!(strip_quotes("\"hello\""), "hello");
    }

    #[test]
    fn strip_quotes_single() {
        assert_eq!(strip_quotes("'hello'"), "hello");
    }

    #[test]
    fn strip_quotes_none() {
        assert_eq!(strip_quotes("hello"), "hello");
    }

    #[test]
    fn strip_quotes_mismatched() {
        assert_eq!(strip_quotes("\"hello'"), "\"hello'");
    }

    #[test]
    fn expand_tilde_plain_path() {
        assert_eq!(
            expand_tilde("/tmp/file.cql"),
            PathBuf::from("/tmp/file.cql")
        );
    }

    #[test]
    fn expand_tilde_home() {
        if let Some(home) = dirs::home_dir() {
            assert_eq!(expand_tilde("~/test.cql"), home.join("test.cql"));
        }
    }

    #[test]
    fn shell_state_initial() {
        let state = ShellState {
            expand: false,
            paging_enabled: true,
            is_tty: false,
            debug: false,
            capture_file: None,
            capture_path: None,
            schema_cache: None,
            shared_keyspace: None,
            colorizer: CqlColorizer::new(false),
        };
        assert!(!state.expand);
        assert!(state.paging_enabled);
        assert!(state.capture_file.is_none());
        assert!(state.capture_path.is_none());
    }

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

    // --- BUG: Shell commands with trailing semicolons ---

    // --- SHOW SESSION tests ---

    #[test]
    fn show_session_parses_uuid() {
        let input = "SHOW SESSION 12345678-1234-1234-1234-123456789abc";
        let upper = input.trim().to_uppercase();
        assert!(upper.starts_with("SHOW SESSION "));
        let uuid_str = input.trim()["SHOW SESSION ".len()..].trim();
        let uuid = uuid::Uuid::parse_str(uuid_str).unwrap();
        assert_eq!(uuid.to_string(), "12345678-1234-1234-1234-123456789abc");
    }

    #[test]
    fn show_session_rejects_invalid_uuid() {
        let uuid_str = "not-a-uuid";
        assert!(uuid::Uuid::parse_str(uuid_str).is_err());
    }

    #[test]
    fn show_session_bare_detected_as_shell_command() {
        assert!(parser::is_shell_command(
            "SHOW SESSION 12345678-1234-1234-1234-123456789abc"
        ));
        assert!(parser::is_shell_command("SHOW SESSION"));
    }

    // --- Shell command semicolon tests ---

    #[test]
    fn shell_command_semicolon_stripped_before_dispatch() {
        // Bug: `DESCRIBE KEYSPACES;` was dispatched with `;` intact,
        // causing describe::execute to receive args="KEYSPACES;" which didn't match.
        let input = "DESCRIBE KEYSPACES;";
        let clean = input.strip_suffix(';').unwrap_or(input).trim_end();
        assert_eq!(clean, "DESCRIBE KEYSPACES");
    }

    #[test]
    fn shell_command_without_semicolon_unchanged() {
        let input = "DESCRIBE KEYSPACES";
        let clean = input.strip_suffix(';').unwrap_or(input).trim_end();
        assert_eq!(clean, "DESCRIBE KEYSPACES");
    }

    #[test]
    fn describe_table_semicolon_stripped() {
        let input = "DESCRIBE TABLE test_ks.events;";
        let clean = input.strip_suffix(';').unwrap_or(input).trim_end();
        assert_eq!(clean, "DESCRIBE TABLE test_ks.events");
        // Verify the args extraction matches what dispatch_input does
        let trimmed = clean.trim();
        let upper = trimmed.to_uppercase();
        assert!(upper.starts_with("DESCRIBE "));
        let args = &trimmed["DESCRIBE ".len()..];
        assert_eq!(args.trim(), "TABLE test_ks.events");
    }

    // --- BUG-4: SOURCE file parsing tests ---

    #[test]
    fn parse_batch_includes_shell_commands() {
        let input = "SELECT 1;\nSHOW VERSION\n";
        let stmts = parser::parse_batch(input);
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0], "SELECT 1");
        assert_eq!(stmts[1], "SHOW VERSION");
    }

    #[test]
    fn parse_batch_shell_command_with_semicolon() {
        let input = "SHOW VERSION;\nSELECT 1;\n";
        let stmts = parser::parse_batch(input);
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0], "SHOW VERSION");
        assert_eq!(stmts[1], "SELECT 1");
    }

    #[test]
    fn source_file_line_by_line_detects_shell_commands() {
        let lines = vec!["CONSISTENCY QUORUM", "SELECT * FROM t;", "SHOW HOST"];
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
        let pasted = "SHOW VERSION\nSELECT 1;\nSHOW HOST";
        let lines: Vec<&str> = pasted.split('\n').collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "SHOW VERSION");
        assert_eq!(lines[1], "SELECT 1;");
        assert_eq!(lines[2], "SHOW HOST");
        assert!(parser::is_shell_command(lines[0].trim()));
        assert!(parser::is_shell_command(lines[2].trim()));
    }

    #[test]
    fn multiline_paste_shell_command_not_concatenated() {
        let pasted = "CAPTURE '/tmp/test.txt'\nSELECT 1;\nCAPTURE OFF";
        let lines: Vec<&str> = pasted.split('\n').collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "CAPTURE '/tmp/test.txt'");
        assert!(parser::is_shell_command(lines[0].trim()));
    }

    // --- print_help tests ---

    #[test]
    fn print_help_contains_documented_commands() {
        let mut buf = Vec::new();
        print_help(&mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Documented shell commands:"));
        assert!(output.contains("CAPTURE"));
        assert!(output.contains("CONSISTENCY"));
        assert!(output.contains("DESCRIBE"));
        assert!(output.contains("EXIT / QUIT"));
        assert!(output.contains("EXPAND"));
        assert!(output.contains("TRACING"));
        assert!(output.contains("COPY TO"));
        assert!(output.contains("CQL statements"));
    }

    #[test]
    fn print_help_contains_all_shell_commands() {
        let mut buf = Vec::new();
        print_help(&mut buf);
        let output = String::from_utf8(buf).unwrap();
        for cmd in [
            "CAPTURE", "CLEAR", "CONSISTENCY", "DEBUG", "DESCRIBE", "EXPAND", "HELP", "LOGIN",
            "PAGING", "SERIAL", "SHOW", "SOURCE", "TRACING", "UNICODE",
        ] {
            assert!(output.contains(cmd), "Missing command: {cmd}");
        }
    }

    // --- print_help_topic tests ---

    #[test]
    fn print_help_topic_known_shell_command() {
        let mut buf = Vec::new();
        print_help_topic("capture", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Help topic: CAPTURE"));
    }

    #[test]
    fn print_help_topic_known_cql_topic() {
        let mut buf = Vec::new();
        print_help_topic("SELECT", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Help topic: SELECT"));
    }

    #[test]
    fn print_help_topic_case_insensitive() {
        let mut buf = Vec::new();
        print_help_topic("consistency", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Help topic: CONSISTENCY"));
    }

    #[test]
    fn print_help_topic_unknown() {
        let mut buf = Vec::new();
        print_help_topic("nonexistent", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("No help topic matching 'nonexistent'"));
    }

    #[test]
    fn print_help_topic_all_shell_commands_recognized() {
        for cmd in [
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
        ] {
            let mut buf = Vec::new();
            print_help_topic(cmd, &mut buf);
            let output = String::from_utf8(buf).unwrap();
            assert!(
                output.contains("Help topic:"),
                "Command '{cmd}' not recognized"
            );
        }
    }

    #[test]
    fn print_help_topic_cql_topics_recognized() {
        for topic in ["INSERT", "UPDATE", "DELETE", "CREATE_TABLE", "DROP_KEYSPACE", "GRANT"] {
            let mut buf = Vec::new();
            print_help_topic(topic, &mut buf);
            let output = String::from_utf8(buf).unwrap();
            assert!(
                output.contains("Help topic:"),
                "CQL topic '{topic}' not recognized"
            );
        }
    }
}

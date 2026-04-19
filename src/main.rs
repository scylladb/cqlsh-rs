//! cqlsh-rs — A Rust re-implementation of the Apache Cassandra cqlsh shell.

use std::io::{self, IsTerminal, Write};

use anyhow::Result;
use clap::{CommandFactory, Parser};

use cqlsh_rs::cli::CliArgs;
use cqlsh_rs::colorizer::CqlColorizer;
use cqlsh_rs::config::{load_config, ColorMode, MergedConfig};
use cqlsh_rs::error;
use cqlsh_rs::formatter;
use cqlsh_rs::parser::{self, ParseResult, StatementParser};
use cqlsh_rs::repl;
use cqlsh_rs::session::CqlSession;
use cqlsh_rs::shell_completions;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CliArgs::parse();

    // Handle shell completion generation if requested
    if let Some(shell) = cli.completions {
        shell_completions::generate(shell);
        return Ok(());
    }

    // Handle man page generation (used by release pipeline)
    if cli.generate_man {
        let cmd = CliArgs::command();
        let man = clap_mangen::Man::new(cmd);
        man.render(&mut io::stdout())?;
        return Ok(());
    }

    if let Err(e) = cli.validate() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    let config = load_config(&cli)?;

    if cli.debug {
        eprintln!("Using CQL driver: scylla-rust-driver");
        eprintln!("Using connect timeout: {} seconds", config.connect_timeout);
        eprintln!("Using request timeout: {} seconds", config.request_timeout);
        eprintln!("Using '{}' encoding", config.encoding);
        eprintln!("Using ssl: {}", config.ssl);
        eprintln!("Debug: resolved host={}, port={}", config.host, config.port);
        eprintln!("Debug: cqlshrc path={}", config.cqlshrc_path.display());
        if let Some(ref v) = config.cqlversion {
            eprintln!("Debug: cqlversion={v}");
        }
        if let Some(v) = config.protocol_version {
            eprintln!("Debug: protocol_version={v}");
        }
        if config.tty {
            eprintln!("Debug: --tty flag set, forcing TTY mode");
        }
    }

    // Connect to the cluster
    let mut session = match CqlSession::connect(&config).await {
        Ok(session) => session,
        Err(e) => {
            eprintln!(
                "Connection error: ('Unable to connect to any servers', \
                 {{'{}:{}'}})",
                config.host, config.port
            );
            if config.debug {
                eprintln!("Debug: {e:?}");
            }
            // Exit code 2 = connection failure (distinct from CQL error = 1)
            std::process::exit(2);
        }
    };

    // Determine whether stdin is a pipe/redirect (non-TTY) and --tty hasn't
    // been set to force interactive mode.
    let stdin_is_pipe = !io::stdin().is_terminal() && !config.tty;

    // Print the connection banner unless stdin is piped/redirected in batch mode.
    // Python cqlsh always prints the banner with -e/-f, only suppresses it when
    // reading from a pipe/redirect without an explicit command.
    let suppress_banner = stdin_is_pipe && config.execute.is_none() && config.file.is_none();
    if !suppress_banner {
        print_banner(&session);
    }

    if config.execute.is_some() || config.file.is_some() {
        // Non-interactive execution mode (-e or -f)
        let exit_code = execute_noninteractive(&mut session, &config).await;
        std::process::exit(exit_code);
    } else if stdin_is_pipe {
        // Stdin pipe mode: read CQL statements from piped/redirected stdin
        let exit_code = execute_stdin(&mut session, &config).await;
        std::process::exit(exit_code);
    } else {
        // Enter interactive REPL (TTY or --tty override)
        repl::run(&mut session, &config).await?;
    }

    Ok(())
}

/// Execute statements in non-interactive mode (-e or -f).
///
/// Returns exit code: 0 on success, 1 on any CQL error.
async fn execute_noninteractive(session: &mut CqlSession, config: &MergedConfig) -> i32 {
    // Resolve color mode: Auto → check if stdout is a terminal
    let color_enabled = match config.color {
        ColorMode::On => true,
        ColorMode::Off => false,
        ColorMode::Auto => io::stdout().is_terminal(),
    };
    let colorizer = CqlColorizer::new(color_enabled);

    if let Some(ref cql_string) = config.execute {
        execute_cql_string(session, config, &colorizer, cql_string).await
    } else if let Some(ref file_path) = config.file {
        execute_cql_file(session, config, &colorizer, file_path).await
    } else {
        0
    }
}

/// Execute CQL statements piped from stdin (non-TTY stdin without -e/-f).
///
/// Returns exit code: 0 on success, 1 on any CQL error.
async fn execute_stdin(session: &mut CqlSession, config: &MergedConfig) -> i32 {
    // When stdin is a pipe stdout is also typically not a terminal.
    let color_enabled = match config.color {
        ColorMode::On => true,
        ColorMode::Off => false,
        ColorMode::Auto => io::stdout().is_terminal(),
    };
    let colorizer = CqlColorizer::new(color_enabled);
    let reader = io::BufReader::new(io::stdin().lock());
    execute_cql_reader(session, config, &colorizer, reader, "<stdin>").await
}

/// Execute a CQL string from the `-e` flag (semicolon-separated statements).
async fn execute_cql_string(
    session: &mut CqlSession,
    config: &MergedConfig,
    colorizer: &CqlColorizer,
    cql_string: &str,
) -> i32 {
    // Python cqlsh accepts `-e "SELECT 1"` without a trailing semicolon.
    // parse_batch silently drops statements that lack one, so normalise here.
    let with_semi;
    let cql_string = {
        let t = cql_string.trim_end();
        if !t.is_empty() && !t.ends_with(';') {
            with_semi = format!("{t};");
            &with_semi
        } else {
            cql_string
        }
    };
    let statements = parser::parse_batch(cql_string);
    let mut had_error = false;
    let mut debug = config.debug;

    for stmt in statements {
        if !execute_single_statement(session, config, colorizer, &mut debug, &stmt).await {
            had_error = true;
        }
    }

    if had_error {
        1
    } else {
        0
    }
}

/// Execute CQL statements from a file (`-f` flag).
async fn execute_cql_file(
    session: &mut CqlSession,
    config: &MergedConfig,
    colorizer: &CqlColorizer,
    file_path: &str,
) -> i32 {
    let file = match std::fs::File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not open '{}': {e}", file_path);
            return 1;
        }
    };
    let reader = io::BufReader::new(file);
    execute_cql_reader(session, config, colorizer, reader, file_path).await
}

/// Execute CQL statements from any `BufRead` source (file or stdin).
///
/// `source_name` is used in I/O error messages.
/// Returns exit code: 0 on success, 1 on any CQL or I/O error.
async fn execute_cql_reader<R: io::BufRead>(
    session: &mut CqlSession,
    config: &MergedConfig,
    colorizer: &CqlColorizer,
    reader: R,
    source_name: &str,
) -> i32 {
    let mut stmt_parser = StatementParser::new();
    let mut had_error = false;
    let mut debug = config.debug;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading '{}': {e}", source_name);
                return 1;
            }
        };

        // Check for shell commands on a fresh line
        let trimmed = line.trim();
        if stmt_parser.is_empty() && !trimmed.is_empty() && parser::is_shell_command(trimmed) {
            let clean = trimmed.strip_suffix(';').unwrap_or(trimmed).trim_end();
            if !execute_single_statement(session, config, colorizer, &mut debug, clean).await {
                had_error = true;
            }
            continue;
        }

        if let ParseResult::Complete(statements) = stmt_parser.feed_line(&line) {
            for stmt in statements {
                if !execute_single_statement(session, config, colorizer, &mut debug, &stmt).await {
                    had_error = true;
                }
            }
        }
    }

    if had_error {
        1
    } else {
        0
    }
}

/// Execute a single CQL statement or shell command in non-interactive mode.
///
/// `debug` is a mutable flag so that DEBUG ON/OFF commands take effect for
/// subsequent statements in the same batch.
///
/// Returns `true` on success, `false` on error.
async fn execute_single_statement(
    session: &mut CqlSession,
    config: &MergedConfig,
    colorizer: &CqlColorizer,
    debug: &mut bool,
    input: &str,
) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return true;
    }

    let upper = trimmed.to_uppercase();

    // Handle DEBUG command (toggle debug output within a batch)
    if upper == "DEBUG" {
        if *debug {
            println!("Debug output is currently enabled. Use DEBUG OFF to disable.");
        } else {
            println!("Debug output is currently disabled. Use DEBUG ON to enable.");
        }
        return true;
    }
    if upper == "DEBUG ON" {
        *debug = true;
        println!("Now printing debug output.");
        return true;
    }
    if upper == "DEBUG OFF" {
        *debug = false;
        println!("Disabled debug output.");
        return true;
    }

    // Handle UNICODE command
    if upper == "UNICODE" {
        println!("Encoding: {}\nDefault encoding: utf-8", config.encoding);
        return true;
    }

    // Handle CONSISTENCY
    if upper == "CONSISTENCY" {
        let cl = session.get_consistency();
        println!("Current consistency level is {cl}.");
        return true;
    }
    if let Some(rest) = upper.strip_prefix("CONSISTENCY ") {
        let level = rest.trim();
        match session.set_consistency_str(level) {
            Ok(()) => println!("Consistency level set to {level}."),
            Err(e) => {
                eprintln!("{e}");
                return false;
            }
        }
        return true;
    }
    if upper == "SERIAL CONSISTENCY" {
        match session.get_serial_consistency() {
            Some(scl) => println!("Current serial consistency level is {scl}."),
            None => println!("Current serial consistency level is SERIAL."),
        }
        return true;
    }
    if let Some(rest) = upper.strip_prefix("SERIAL CONSISTENCY ") {
        let level = rest.trim();
        match session.set_serial_consistency_str(level) {
            Ok(()) => println!("Serial consistency level set to {level}."),
            Err(e) => {
                eprintln!("{e}");
                return false;
            }
        }
        return true;
    }
    if upper == "TRACING OFF" || upper == "TRACING" {
        session.set_tracing(false);
        println!("Disabled tracing.");
        return true;
    }
    if upper == "TRACING ON" {
        session.set_tracing(true);
        println!("Now tracing requests.");
        return true;
    }
    if upper == "SHOW VERSION" {
        println!("[cqlsh {}]", env!("CARGO_PKG_VERSION"));
        return true;
    }
    if upper == "SHOW HOST" {
        println!("Connected to: {}", session.connection_display);
        return true;
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
        match cqlsh_rs::describe::execute(session, args, &mut buf).await {
            Ok(()) => {
                let _ = io::stdout().write_all(&buf);
            }
            Err(e) => {
                eprintln!("Error: {e}");
                return false;
            }
        }
        return true;
    }

    // Skip commands that don't make sense in non-interactive mode
    if upper == "QUIT"
        || upper == "EXIT"
        || upper == "CLEAR"
        || upper == "CLS"
        || upper == "HELP"
        || upper == "?"
        || upper.starts_with("HELP ")
        || upper == "EXPAND"
        || upper == "EXPAND ON"
        || upper == "EXPAND OFF"
        || upper == "PAGING"
        || upper == "PAGING ON"
        || upper == "PAGING OFF"
        || upper.starts_with("PAGING ")
        || upper == "CAPTURE"
        || upper == "CAPTURE OFF"
        || upper.starts_with("CAPTURE ")
        || upper == "LOGIN"
        || upper.starts_with("LOGIN ")
    {
        // Silently ignore interactive-only commands
        return true;
    }

    // Execute as CQL statement
    match session.execute(trimmed).await {
        Ok(result) => {
            // Print warnings to stderr
            for warning in &result.warnings {
                let msg = format!("Warnings: {warning}");
                eprintln!("{}", colorizer.colorize_warning(&msg));
            }

            // Print results directly to stdout (no pager)
            if !result.columns.is_empty() {
                let mut buf = Vec::new();
                formatter::print_tabular(&result, colorizer, &mut buf);
                let _ = io::stdout().write_all(&buf);
            }

            // Print trace info if tracing is enabled
            if session.is_tracing_enabled() {
                if let Some(trace_id) = result.tracing_id {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    match session.get_trace_session(trace_id).await {
                        Ok(Some(trace)) => {
                            let mut buf = Vec::new();
                            formatter::print_trace(&trace, colorizer, &mut buf);
                            let _ = io::stdout().write_all(&buf);
                        }
                        Ok(None) => {
                            eprintln!(
                                "Trace {trace_id} not yet available. Use SHOW SESSION {trace_id} to view later."
                            );
                        }
                        Err(e) => {
                            eprintln!("Error fetching trace: {e}");
                        }
                    }
                }
            }

            true
        }
        Err(e) => {
            eprintln!("{}", error::format_error_colored(&e, colorizer));
            if *debug {
                eprintln!("Debug: {e:?}");
            }
            false
        }
    }
}

/// Print the cqlsh connection banner matching Python cqlsh output.
///
/// Detects ScyllaDB vs Apache Cassandra and shows the appropriate version.
fn print_banner(session: &CqlSession) {
    let cluster_name = session.cluster_name.as_deref().unwrap_or("Unknown Cluster");
    let cql_version = session.cql_version.as_deref().unwrap_or("unknown");

    println!(
        "Connected to {} at {}.",
        cluster_name, session.connection_display
    );

    let server_info = if let Some(scylla_ver) = &session.scylla_version {
        format!("Scylla {scylla_ver}")
    } else {
        let release = session.release_version.as_deref().unwrap_or("unknown");
        format!("Cassandra {release}")
    };

    println!(
        "[cqlsh {} | {} | CQL spec {}]",
        env!("CARGO_PKG_VERSION"),
        server_info,
        cql_version
    );
    println!("Use HELP for help.");
}

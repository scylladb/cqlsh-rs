//! Non-interactive CQL execution engine with injectable output writer.

use std::io::{self, BufRead, Write};

use crate::colorizer::CqlColorizer;
use crate::config::MergedConfig;
use crate::parser::{self, ParseResult, StatementParser};
use crate::session::CqlSession;
use crate::{describe, error, formatter};

/// Execute a CQL string from the `-e` flag (semicolon-separated statements).
///
/// All output is written to `writer`; errors go to stderr.
/// Returns exit code: 0 on success, 1 on any CQL error.
pub async fn execute_cql_string(
    session: &mut CqlSession,
    config: &MergedConfig,
    colorizer: &CqlColorizer,
    cql_string: &str,
    writer: &mut dyn Write,
) -> i32 {
    // Python cqlsh accepts `-e "SELECT 1"` without a trailing semicolon.
    let with_semi;
    let cql_string = {
        let t = cql_string.trim_end();
        // Python cqlsh accepts `-e "SELECT 1"` without a trailing semicolon.
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
        if !execute_single_statement(
            session, config, colorizer, &mut debug, &stmt, None, 0, writer,
        )
        .await
        {
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
///
/// Returns exit code: 0 on success, 1 on any CQL or I/O error.
pub async fn execute_cql_file(
    session: &mut CqlSession,
    config: &MergedConfig,
    colorizer: &CqlColorizer,
    file_path: &str,
    writer: &mut dyn Write,
) -> i32 {
    let file = match std::fs::File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not open '{}': {e}", file_path);
            return 1;
        }
    };
    let reader = io::BufReader::new(file);
    execute_cql_reader(session, config, colorizer, reader, file_path, writer).await
}

/// Execute CQL statements from any `BufRead` source (file or stdin).
///
/// Returns exit code: 0 on success, 1 on any CQL or I/O error.
pub async fn execute_cql_reader<R: io::BufRead>(
    session: &mut CqlSession,
    config: &MergedConfig,
    colorizer: &CqlColorizer,
    reader: R,
    source_name: &str,
    writer: &mut dyn Write,
) -> i32 {
    let mut stmt_parser = StatementParser::new();
    let mut had_error = false;
    let mut debug = config.debug;
    let mut line_number: usize = 0;
    let mut stmt_start_line: usize = 1;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading '{}': {e}", source_name);
                return 1;
            }
        };
        line_number += 1;

        if stmt_parser.is_empty() {
            stmt_start_line = line_number;
        }

        let trimmed = line.trim();
        if stmt_parser.is_empty() && !trimmed.is_empty() && parser::is_shell_command(trimmed) {
            let clean = trimmed.strip_suffix(';').unwrap_or(trimmed).trim_end();
            if !execute_single_statement(
                session,
                config,
                colorizer,
                &mut debug,
                clean,
                Some(source_name),
                stmt_start_line,
                writer,
            )
            .await
            {
                had_error = true;
            }
            continue;
        }

        if let ParseResult::Complete(statements) = stmt_parser.feed_line(&line) {
            for stmt in statements {
                if !execute_single_statement(
                    session,
                    config,
                    colorizer,
                    &mut debug,
                    &stmt,
                    Some(source_name),
                    stmt_start_line,
                    writer,
                )
                .await
                {
                    had_error = true;
                }
            }
            stmt_start_line = line_number + 1;
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
/// Output is written to `writer`; errors and warnings are written to stderr.
/// `debug` is mutable so that `DEBUG ON/OFF` affects subsequent statements.
///
/// Returns `true` on success, `false` on error.
#[allow(clippy::too_many_arguments)]
pub fn execute_single_statement<'a>(
    session: &'a mut CqlSession,
    config: &'a MergedConfig,
    colorizer: &'a CqlColorizer,
    debug: &'a mut bool,
    input: &'a str,
    source_name: Option<&'a str>,
    line_number: usize,
    writer: &'a mut dyn Write,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + 'a>> {
    Box::pin(async move {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return true;
        }

        let upper = trimmed.to_uppercase();

        if upper == "DEBUG" {
            if *debug {
                let _ = writeln!(
                    writer,
                    "Debug output is currently enabled. Use DEBUG OFF to disable."
                );
            } else {
                let _ = writeln!(
                    writer,
                    "Debug output is currently disabled. Use DEBUG ON to enable."
                );
            }
            return true;
        }
        if upper == "DEBUG ON" {
            *debug = true;
            let _ = writeln!(writer, "Now printing debug output.");
            return true;
        }
        if upper == "DEBUG OFF" {
            *debug = false;
            let _ = writeln!(writer, "Disabled debug output.");
            return true;
        }

        if upper == "UNICODE" {
            let _ = writeln!(
                writer,
                "Encoding: {}\nDefault encoding: utf-8",
                config.encoding
            );
            return true;
        }

        if upper == "CONSISTENCY" {
            let cl = session.get_consistency();
            let _ = writeln!(writer, "Current consistency level is {cl}.");
            return true;
        }
        if let Some(rest) = upper.strip_prefix("CONSISTENCY ") {
            let level = rest.trim();
            match session.set_consistency_str(level) {
                Ok(()) => {
                    let _ = writeln!(writer, "Consistency level set to {level}.");
                }
                Err(e) => {
                    eprintln!("{e}");
                    return false;
                }
            }
            return true;
        }
        if upper == "SERIAL CONSISTENCY" {
            match session.get_serial_consistency() {
                Some(scl) => {
                    let _ = writeln!(writer, "Current serial consistency level is {scl}.");
                }
                None => {
                    let _ = writeln!(writer, "Current serial consistency level is SERIAL.");
                }
            }
            return true;
        }
        if let Some(rest) = upper.strip_prefix("SERIAL CONSISTENCY ") {
            let level = rest.trim();
            match session.set_serial_consistency_str(level) {
                Ok(()) => {
                    let _ = writeln!(writer, "Serial consistency level set to {level}.");
                }
                Err(e) => {
                    eprintln!("{e}");
                    return false;
                }
            }
            return true;
        }
        if upper == "TRACING OFF" || upper == "TRACING" {
            session.set_tracing(false);
            let _ = writeln!(writer, "Disabled tracing.");
            return true;
        }
        if upper == "TRACING ON" {
            session.set_tracing(true);
            let _ = writeln!(writer, "Now tracing requests.");
            return true;
        }
        if upper == "SHOW VERSION" {
            let _ = writeln!(writer, "[cqlsh {}]", env!("CARGO_PKG_VERSION"));
            return true;
        }
        if upper == "SHOW HOST" {
            let _ = writeln!(writer, "Connected to: {}", session.connection_display);
            return true;
        }

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
                Ok(()) => {
                    let _ = writer.write_all(&buf);
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    return false;
                }
            }
            return true;
        }

        if upper.starts_with("SOURCE ") {
            let path = trimmed["SOURCE ".len()..].trim();
            let path = strip_quotes(path);
            if config.no_file_io {
                eprintln!("File I/O is disabled (--no-file-io).");
                return true;
            }
            let expanded = expand_tilde(path);
            let file = match std::fs::File::open(&expanded) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Could not open '{}': {e}", expanded.display());
                    return false;
                }
            };
            let reader = io::BufReader::new(file);
            let source_name_str = expanded.display().to_string();
            let mut stmt_parser = StatementParser::new();
            let mut had_error = false;
            let mut src_line_number: usize = 0;
            let mut src_stmt_start: usize = 1;
            for line_result in reader.lines() {
                let line: String = match line_result {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("Error reading '{}': {e}", source_name_str);
                        return false;
                    }
                };
                src_line_number += 1;
                if stmt_parser.is_empty() {
                    src_stmt_start = src_line_number;
                }
                let ltrimmed = line.trim();
                if stmt_parser.is_empty()
                    && !ltrimmed.is_empty()
                    && parser::is_shell_command(ltrimmed)
                {
                    let clean = ltrimmed.strip_suffix(';').unwrap_or(ltrimmed).trim_end();
                    if !execute_single_statement(
                        session,
                        config,
                        colorizer,
                        debug,
                        clean,
                        Some(&source_name_str),
                        src_stmt_start,
                        writer,
                    )
                    .await
                    {
                        had_error = true;
                    }
                    continue;
                }
                if let ParseResult::Complete(statements) = stmt_parser.feed_line(&line) {
                    for stmt in statements {
                        if !execute_single_statement(
                            session,
                            config,
                            colorizer,
                            debug,
                            &stmt,
                            Some(&source_name_str),
                            src_stmt_start,
                            writer,
                        )
                        .await
                        {
                            had_error = true;
                        }
                    }
                    src_stmt_start = src_line_number + 1;
                }
            }
            return !had_error;
        }
        if upper == "SOURCE" {
            eprintln!("SOURCE requires a file path argument.");
            return true;
        }

        if upper == "CLEAR" || upper == "CLS" {
            let _ = write!(writer, "\x1B[2J\x1B[1;1H");
            return true;
        }

        if upper == "LOGIN" {
            eprintln!("Usage: LOGIN <username> [<password>]");
            return false;
        }
        if upper.starts_with("LOGIN ") {
            let args = trimmed["LOGIN ".len()..].trim();
            let parts: Vec<&str> = args.splitn(2, char::is_whitespace).collect();
            let new_user = parts[0].to_string();
            let new_pass = if parts.len() > 1 {
                Some(parts[1].trim_matches('\'').to_string())
            } else {
                None
            };
            let mut new_config = config.clone();
            new_config.username = Some(new_user);
            new_config.password = new_pass;
            match crate::session::CqlSession::connect(&new_config).await {
                Ok(new_session) => {
                    *session = new_session;
                }
                Err(e) => {
                    eprintln!("{}", error::format_error_colored(&e, colorizer));
                    return false;
                }
            }
            return true;
        }

        if upper == "QUIT"
            || upper == "EXIT"
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
        {
            return true;
        }

        match session.execute(trimmed).await {
            Ok(result) => {
                for warning in &result.warnings {
                    let msg = format!("Warnings: {warning}");
                    eprintln!("{}", colorizer.colorize_warning(&msg));
                }

                if !result.columns.is_empty() {
                    let mut buf = Vec::new();
                    formatter::print_tabular(&result, colorizer, &mut buf);
                    let _ = writer.write_all(&buf);
                }

                if session.is_tracing_enabled() && !upper.contains("SYSTEM_TRACES") {
                    if let Some(trace_id) = result.tracing_id {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        match session.get_trace_session(trace_id).await {
                            Ok(Some(trace)) => {
                                let mut buf = Vec::new();
                                formatter::print_trace(&trace, colorizer, &mut buf);
                                let _ = writer.write_all(&buf);
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
                let err_msg = error::format_error_colored(&e, colorizer);
                if let Some(src) = source_name {
                    eprintln!("{src}:{line_number}:{err_msg}");
                } else {
                    eprintln!("{err_msg}");
                }
                if *debug {
                    eprintln!("Debug: {e:?}");
                }
                false
            }
        }
    })
}

fn strip_quotes(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn expand_tilde(path: &str) -> std::path::PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    std::path::PathBuf::from(path)
}

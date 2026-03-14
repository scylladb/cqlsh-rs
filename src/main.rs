//! cqlsh-rs — A Rust re-implementation of the Apache Cassandra cqlsh shell.

use anyhow::Result;
use clap::Parser;

use cqlsh_rs::cli::CliArgs;
use cqlsh_rs::config::load_config;
use cqlsh_rs::runner;
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

    if let Err(e) = cli.validate() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    let config = load_config(&cli)?;

    if cli.debug {
        eprintln!("Debug: resolved host={}, port={}", config.host, config.port);
        eprintln!("Debug: cqlshrc path={}", config.cqlshrc_path.display());
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
            std::process::exit(1);
        }
    };

    // Execute mode (-e): run statement and exit
    if let Some(ref statement) = config.execute {
        if let Err(e) = runner::execute_statement(&mut session, statement).await {
            print_error(&e, config.debug);
            std::process::exit(1);
        }
        return Ok(());
    }

    // File mode (-f): run file and exit
    if let Some(ref path) = config.file {
        if let Err(e) = runner::execute_file(&mut session, path).await {
            print_error(&e, config.debug);
            std::process::exit(1);
        }
        return Ok(());
    }

    // Interactive mode — print banner
    print_banner(&session);
    eprintln!(
        "Interactive REPL not yet implemented. \
         Connected to {}:{} successfully.",
        config.host, config.port
    );

    Ok(())
}

/// Print an error message. In normal mode, show only the root cause.
/// In debug mode, show the full error chain.
fn print_error(error: &anyhow::Error, debug: bool) {
    if debug {
        eprintln!("{error:#}");
    } else {
        // Walk to the innermost (root cause) error
        eprintln!("{}", error.root_cause());
    }
}

/// Print the cqlsh connection banner matching Python cqlsh output.
fn print_banner(session: &CqlSession) {
    let cluster_name = session.cluster_name.as_deref().unwrap_or("Unknown Cluster");
    let cql_version = session.cql_version.as_deref().unwrap_or("unknown");
    let release_version = session.release_version.as_deref().unwrap_or("unknown");

    println!(
        "Connected to {} at {}.",
        cluster_name, session.connection_display
    );
    println!(
        "[cqlsh {} | Cassandra {} | CQL spec {}]",
        env!("CARGO_PKG_VERSION"),
        release_version,
        cql_version
    );
    println!("Use HELP for help.");
}

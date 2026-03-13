//! cqlsh-rs — A Rust re-implementation of the Apache Cassandra cqlsh shell.

use anyhow::Result;
use clap::Parser;

use cqlsh_rs::cli::CliArgs;
use cqlsh_rs::config::load_config;
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

    // Print connection banner
    print_banner(&session);

    if config.execute.is_some() || config.file.is_some() {
        eprintln!(
            "Non-interactive mode not yet implemented. \
             Connected to {}:{} successfully.",
            config.host, config.port
        );
    } else {
        // Enter interactive REPL
        repl::run(&mut session, &config).await?;
    }

    Ok(())
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

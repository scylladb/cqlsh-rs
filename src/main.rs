//! cqlsh-rs — A Rust re-implementation of the Apache Cassandra cqlsh shell.

mod cli;
#[allow(dead_code)]
mod config;
mod shell_completions;

use anyhow::Result;
use clap::Parser;

use cli::CliArgs;
use config::load_config;

fn main() -> Result<()> {
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

    // For now, print resolved config and exit.
    // Future phases will add driver connection and REPL.
    if config.execute.is_some() || config.file.is_some() {
        eprintln!(
            "Non-interactive mode not yet implemented. \
             Will connect to {}:{} in future phases.",
            config.host, config.port
        );
    } else {
        eprintln!(
            "Interactive mode not yet implemented. \
             Will connect to {}:{} in future phases.",
            config.host, config.port
        );
    }

    Ok(())
}

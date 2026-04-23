//! cqlsh-rs — A Rust re-implementation of the Apache Cassandra cqlsh shell.

use std::io::{self, IsTerminal};

use anyhow::Result;
use clap::{CommandFactory, Parser};

use cqlsh_rs::cli::CliArgs;
use cqlsh_rs::colorizer::CqlColorizer;
use cqlsh_rs::config::{load_config, ColorMode, MergedConfig};
use cqlsh_rs::executor;
use cqlsh_rs::repl;
use cqlsh_rs::session::CqlSession;
use cqlsh_rs::shell_completions;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CliArgs::parse();

    if let Some(shell) = cli.completions {
        shell_completions::generate(shell);
        return Ok(());
    }

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
            std::process::exit(2);
        }
    };

    if !session.check_schema_agreement().await {
        eprintln!(
            "\nWarning: schema version mismatch detected; check the schema versions of your \
                   nodes in system.local and system.peers.\n"
        );
    }

    let stdin_is_pipe = !io::stdin().is_terminal() && !config.tty;

    // Python cqlsh always prints the banner with -e/-f, only suppresses it when
    // reading from a pipe/redirect without an explicit command.
    let suppress_banner = stdin_is_pipe && config.execute.is_none() && config.file.is_none();
    if !suppress_banner {
        print_banner(&session);
    }

    if let Some(ref requested) = config.cqlversion {
        let actual = session.cql_version.as_deref().unwrap_or("unknown");
        if requested != actual {
            eprintln!(
                "Warning: --cqlversion {requested} requested, but the server reports CQL spec {actual}. \
                 The scylla driver does not support overriding the CQL version."
            );
        }
    }
    if config.protocol_version.is_some() {
        eprintln!(
            "Warning: --protocol-version is accepted for CLI compatibility but the scylla \
             driver auto-negotiates the native protocol version."
        );
    }

    if config.execute.is_some() || config.file.is_some() {
        let exit_code = execute_noninteractive(&mut session, &config).await;
        std::process::exit(exit_code);
    } else if stdin_is_pipe {
        let exit_code = execute_stdin(&mut session, &config).await;
        std::process::exit(exit_code);
    } else {
        repl::run(&mut session, &config).await?;
    }

    Ok(())
}

async fn execute_noninteractive(session: &mut CqlSession, config: &MergedConfig) -> i32 {
    let color_enabled = match config.color {
        ColorMode::On => true,
        ColorMode::Off => false,
        ColorMode::Auto => io::stdout().is_terminal(),
    };
    let colorizer = CqlColorizer::new(color_enabled);
    let stdout = &mut io::stdout();

    if let Some(ref cql_string) = config.execute {
        executor::execute_cql_string(session, config, &colorizer, cql_string, stdout).await
    } else if let Some(ref file_path) = config.file {
        executor::execute_cql_file(session, config, &colorizer, file_path, stdout).await
    } else {
        0
    }
}

async fn execute_stdin(session: &mut CqlSession, config: &MergedConfig) -> i32 {
    let color_enabled = match config.color {
        ColorMode::On => true,
        ColorMode::Off => false,
        ColorMode::Auto => io::stdout().is_terminal(),
    };
    let colorizer = CqlColorizer::new(color_enabled);
    let reader = io::BufReader::new(io::stdin().lock());
    let stdout = &mut io::stdout();
    executor::execute_cql_reader(session, config, &colorizer, reader, "<stdin>", stdout).await
}

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

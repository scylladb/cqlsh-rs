//! cqlsh-rs library — exposes modules for benchmarks and integration tests.

pub mod cli;
pub mod colorizer;
pub mod completer;
pub mod config;
pub mod copy;
pub mod cql_lexer;
pub mod describe;
pub mod driver;
pub mod error;
pub mod executor;
pub mod formatter;
pub mod pager;
pub mod parser;
pub mod repl;
pub mod schema_cache;
pub mod session;
pub mod shell_completions;

use std::io::Write;

use anyhow::Result;

/// Execute a CQL string against a remote host and return the captured stdout output.
///
/// Connects to the given host/port, optionally uses `keyspace`, then runs `cql`
/// as if it were passed to `-e`. This runs in-process so `cargo tarpaulin` can
/// measure coverage for all code paths exercised.
pub async fn run_cql_in_process(
    host: &str,
    port: u16,
    keyspace: Option<&str>,
    cql: &str,
) -> Result<String> {
    use config::{ColorMode, CqlshrcConfig, MergedConfig};
    use std::path::PathBuf;

    let mut config = MergedConfig {
        host: host.to_string(),
        port,
        username: None,
        password: None,
        keyspace: keyspace.map(str::to_string),
        ssl: false,
        color: ColorMode::Off,
        debug: false,
        tty: false,
        no_file_io: false,
        no_compact: false,
        disable_history: true,
        execute: None,
        file: None,
        connect_timeout: 10,
        request_timeout: 30,
        encoding: "utf-8".to_string(),
        cqlversion: None,
        protocol_version: None,
        consistency_level: None,
        serial_consistency_level: None,
        browser: None,
        secure_connect_bundle: None,
        fetch_size: 100,
        cqlshrc_path: PathBuf::from("/dev/null"),
        cqlshrc: CqlshrcConfig::default(),
    };

    let mut session = session::CqlSession::connect(&config).await?;

    if let Some(ks) = keyspace {
        session.use_keyspace(ks).await?;
        config.keyspace = Some(ks.to_string());
    }

    let colorizer = colorizer::CqlColorizer::new(false);
    let mut output: Vec<u8> = Vec::new();
    executor::execute_cql_string(&mut session, &config, &colorizer, cql, &mut output).await;

    let _ = output.flush();
    Ok(String::from_utf8_lossy(&output).into_owned())
}

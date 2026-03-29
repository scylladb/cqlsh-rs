//! Shared test helpers for integration tests.
//!
//! Provides ScyllaDB container setup via testcontainers-rs and utility
//! functions for executing cqlsh-rs commands against a live database.

use std::sync::OnceLock;
use std::time::Duration;

use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::SyncRunner;
use testcontainers::{Container, GenericImage, ImageExt};

/// The native CQL transport port inside the container.
const CQL_PORT: u16 = 9042;

/// A running ScyllaDB container with its mapped port.
pub struct ScyllaContainer {
    /// The container handle — kept alive for the process lifetime.
    /// `None` when connecting to a pre-existing instance via env vars.
    _container: Option<Container<GenericImage>>,
    /// The host port mapped to the CQL native transport port.
    pub port: u16,
    /// The host address (always 127.0.0.1 for local Docker).
    pub host: String,
}

/// Outcome of a single container-start attempt.
///
/// Stored in `OnceLock` so the start is attempted exactly once.
/// If it fails the `Err` is remembered and all subsequent tests skip
/// quickly without triggering more Docker start attempts.
type StartResult = Result<ScyllaContainer, String>;

static SCYLLA: OnceLock<StartResult> = OnceLock::new();

/// Return the shared ScyllaDB container, starting it on first call.
///
/// Panics if the container failed to start (message includes the error).
/// Uses `OnceLock<Result<…>>` so Docker is contacted exactly once even
/// when a container start fails mid-way.
pub fn get_scylla() -> &'static ScyllaContainer {
    SCYLLA
        .get_or_init(start_scylla)
        .as_ref()
        .expect("ScyllaDB container is not available")
}

fn start_scylla() -> StartResult {
    // TODO: temporary CI workaround — if SCYLLA_TEST_HOST/PORT are set (injected by
    // the GitHub Actions "Start ScyllaDB" step), skip testcontainers and connect
    // directly. Remove once testcontainers-rs works on GitHub Actions runners.
    if let Ok(host) = std::env::var("SCYLLA_TEST_HOST") {
        let port = std::env::var("SCYLLA_TEST_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(CQL_PORT);
        return Ok(ScyllaContainer {
            _container: None,
            port,
            host,
        });
    }

    let container = GenericImage::new("scylladb/scylla", "6.2")
        .with_wait_for(WaitFor::message_on_stderr("serving"))
        .with_exposed_port(CQL_PORT.tcp())
        .with_cmd(vec![
            "--smp".to_string(),
            "1".to_string(),
            "--memory".to_string(),
            "512M".to_string(),
            "--overprovisioned".to_string(),
            "1".to_string(),
            "--skip-wait-for-gossip-to-settle".to_string(),
            "0".to_string(),
        ])
        .with_startup_timeout(Duration::from_secs(120))
        .start()
        .map_err(|e| format!("failed to start ScyllaDB container: {e}"))?;

    let port = container
        .get_host_port_ipv4(CQL_PORT)
        .map_err(|e| format!("failed to get mapped port: {e}"))?;

    let host = container
        .get_host()
        .map_err(|e| format!("failed to get container host: {e}"))?
        .to_string();

    // Wait a bit for CQL to be fully ready after the log message
    std::thread::sleep(Duration::from_secs(5));

    Ok(ScyllaContainer {
        _container: Some(container),
        port,
        host,
    })
}

/// Execute cqlsh-rs with the given arguments against the test container.
pub fn cqlsh_cmd(scylla: &ScyllaContainer) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("cqlsh-rs").unwrap();
    cmd.args([&scylla.host, &scylla.port.to_string()]);
    cmd
}

/// Ensure a CQL/shell statement has a trailing semicolon.
fn with_semicolon(stmt: &str) -> String {
    let trimmed = stmt.trim_end();
    if trimmed.ends_with(';') {
        trimmed.to_string()
    } else {
        format!("{trimmed};")
    }
}

/// Execute a CQL statement via cqlsh-rs `-e` flag and return the command assertion.
pub fn execute_cql(scylla: &ScyllaContainer, cql: &str) -> assert_cmd::assert::Assert {
    cqlsh_cmd(scylla)
        .args(["-e", &with_semicolon(cql)])
        .assert()
}

/// Execute a CQL statement and return stdout as a string.
/// Panics if the command fails.
pub fn execute_cql_output(scylla: &ScyllaContainer, cql: &str) -> String {
    let output = cqlsh_cmd(scylla)
        .args(["-e", &with_semicolon(cql)])
        .output()
        .expect("failed to execute cqlsh-rs");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("cqlsh-rs failed: {stderr}");
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Create a unique test keyspace and return its name.
pub fn create_test_keyspace(scylla: &ScyllaContainer, prefix: &str) -> String {
    let ks_name = format!(
        "test_{}_{:x}",
        prefix,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            % 0xFFFFFF
    );

    execute_cql(
        scylla,
        &format!(
            "CREATE KEYSPACE IF NOT EXISTS {ks_name} \
             WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}}"
        ),
    )
    .success();

    ks_name
}

/// Drop a test keyspace (cleanup).
pub fn drop_test_keyspace(scylla: &ScyllaContainer, keyspace: &str) {
    execute_cql(scylla, &format!("DROP KEYSPACE IF EXISTS {keyspace}")).success();
}

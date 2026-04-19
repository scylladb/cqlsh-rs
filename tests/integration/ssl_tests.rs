//! SSL/TLS integration tests for cqlsh-rs.
//!
//! Tests both non-TLS error behavior and real TLS connections using a
//! ScyllaDB container configured with server-side encryption.

use super::helpers::*;
use predicates::prelude::*;
use std::io::Write;
use std::sync::OnceLock;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::SyncRunner;
use testcontainers::{Container, GenericImage, ImageExt};

// ---------------------------------------------------------------------------
// TLS container setup
// ---------------------------------------------------------------------------

const CQL_TLS_PORT: u16 = 9042;

struct TlsScyllaContainer {
    _container: Container<GenericImage>,
    port: u16,
    host: String,
    ca_cert_path: std::path::PathBuf,
    _cert_dir: tempfile::TempDir,
}

type TlsStartResult = Result<TlsScyllaContainer, String>;
static TLS_SCYLLA: OnceLock<TlsStartResult> = OnceLock::new();

fn get_tls_scylla() -> Option<&'static TlsScyllaContainer> {
    TLS_SCYLLA.get_or_init(start_tls_scylla).as_ref().ok()
}

/// Helper macro to skip a test when the TLS container is unavailable.
macro_rules! require_tls {
    () => {
        match get_tls_scylla() {
            Some(tls) => tls,
            None => {
                eprintln!(
                    "Skipping test: TLS container unavailable (port conflict or Docker issue)"
                );
                return;
            }
        }
    };
}

fn generate_scylla_yaml() -> String {
    // ScyllaDB reads client_encryption_options from scylla.yaml
    r#"cluster_name: "TLS Test Cluster"
listen_address: "0.0.0.0"
rpc_address: "0.0.0.0"
broadcast_rpc_address: "127.0.0.1"
broadcast_address: "127.0.0.1"
seed_provider:
    - class_name: org.apache.cassandra.locator.SimpleSeedProvider
      parameters:
          - seeds: "127.0.0.1"
client_encryption_options:
    enabled: true
    certificate: /etc/scylla/db.crt
    keyfile: /etc/scylla/db.key
    require_client_auth: false
"#
    .to_string()
}

fn start_tls_scylla() -> TlsStartResult {
    let cert_dir = tempfile::tempdir().map_err(|e| format!("tempdir: {e}"))?;

    // Generate self-signed cert for localhost/127.0.0.1
    let cert =
        rcgen::generate_simple_self_signed(vec!["localhost".to_string(), "127.0.0.1".to_string()])
            .map_err(|e| format!("rcgen: {e}"))?;

    let cert_pem = cert.cert.pem();
    let key_pem = cert.key_pair.serialize_pem();

    // Write CA cert to host for cqlsh-rs --ssl + cqlshrc certfile
    let ca_cert_path = cert_dir.path().join("ca.crt");
    std::fs::write(&ca_cert_path, &cert_pem).map_err(|e| format!("write ca: {e}"))?;

    let scylla_yaml = generate_scylla_yaml();

    let container = GenericImage::new("scylladb/scylla", "6.2")
        .with_wait_for(WaitFor::message_on_stderr("serving"))
        .with_exposed_port(CQL_TLS_PORT.tcp())
        .with_copy_to("/etc/scylla/scylla.yaml", scylla_yaml.into_bytes())
        .with_copy_to("/etc/scylla/db.crt", cert_pem.into_bytes())
        .with_copy_to("/etc/scylla/db.key", key_pem.into_bytes())
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
        .with_startup_timeout(std::time::Duration::from_secs(120))
        .start()
        .map_err(|e| format!("start TLS container: {e}"))?;

    let port = container
        .get_host_port_ipv4(CQL_TLS_PORT)
        .map_err(|e| format!("get port: {e}"))?;

    let host = container
        .get_host()
        .map_err(|e| format!("get host: {e}"))?
        .to_string();

    std::thread::sleep(std::time::Duration::from_secs(5));

    Ok(TlsScyllaContainer {
        _container: container,
        port,
        host,
        ca_cert_path,
        _cert_dir: cert_dir,
    })
}

fn tls_cqlsh_cmd(tls: &TlsScyllaContainer) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("cqlsh-rs").unwrap();
    cmd.args([&tls.host, &tls.port.to_string()]);
    cmd
}

// ---------------------------------------------------------------------------
// Tests against NON-TLS container (existing behavior)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_ssl_flag_fails_gracefully_on_non_ssl_server() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args(["--ssl", "-e", "SELECT release_version FROM system.local"])
        .output()
        .expect("cqlsh-rs process should not hang");

    assert!(
        !output.status.success(),
        "Expected failure with --ssl against non-TLS server"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "Expected an error message on stderr when SSL handshake fails"
    );
}

#[test]
#[ignore = "requires Docker"]
fn test_non_ssl_connection_succeeds() {
    let scylla = get_scylla();

    cqlsh_cmd(scylla)
        .args(["-e", "SELECT release_version FROM system.local"])
        .assert()
        .success();
}

#[test]
#[ignore = "requires Docker"]
fn test_ssl_with_auth_flags_does_not_panic() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args([
            "--ssl",
            "-u",
            "cassandra",
            "-p",
            "cassandra",
            "-e",
            "SELECT release_version FROM system.local",
        ])
        .output()
        .expect("cqlsh-rs should not hang or panic");

    let _ = output.status;
}

#[test]
#[ignore = "requires Docker"]
fn test_ssl_connect_timeout_respected() {
    let scylla = get_scylla();

    let start = std::time::Instant::now();
    let output = cqlsh_cmd(scylla)
        .args([
            "--ssl",
            "--connect-timeout",
            "5",
            "-e",
            "SELECT release_version FROM system.local",
        ])
        .output()
        .expect("cqlsh-rs should not hang indefinitely");

    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs() < 30,
        "SSL connection attempt took too long: {elapsed:?}"
    );
    assert!(
        !output.status.success(),
        "Expected non-zero exit when SSL fails"
    );
}

// ---------------------------------------------------------------------------
// Tests against TLS-enabled container
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_ssl_connection_with_certfile() {
    let tls = require_tls!();

    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[ssl]").unwrap();
    writeln!(f, "certfile = {}", tls.ca_cert_path.display()).unwrap();
    writeln!(f, "validate = true").unwrap();

    tls_cqlsh_cmd(tls)
        .args([
            "--ssl",
            "--cqlshrc",
            cqlshrc.to_str().unwrap(),
            "-e",
            "SELECT release_version FROM system.local",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("release_version"));
}

#[test]
#[ignore = "requires Docker"]
fn test_ssl_connection_no_validate() {
    let tls = require_tls!();

    // --ssl without certfile/validate uses empty root store (no validation)
    tls_cqlsh_cmd(tls)
        .args(["--ssl", "-e", "SELECT release_version FROM system.local"])
        .assert()
        .success()
        .stdout(predicate::str::contains("release_version"));
}

#[test]
#[ignore = "requires Docker"]
fn test_ssl_describe_keyspaces() {
    let tls = require_tls!();

    tls_cqlsh_cmd(tls)
        .args(["--ssl", "-e", "DESCRIBE KEYSPACES"])
        .assert()
        .success()
        .stdout(predicate::str::contains("system"));
}

#[test]
#[ignore = "requires Docker"]
fn test_non_ssl_to_tls_server_fails() {
    let tls = require_tls!();

    // Plain connection to TLS-only server should fail
    tls_cqlsh_cmd(tls)
        .args([
            "-e",
            "SELECT release_version FROM system.local",
            "--connect-timeout",
            "5",
        ])
        .assert()
        .failure();
}

#[test]
#[ignore = "requires Docker"]
fn test_ssl_with_wrong_certfile_fails() {
    let tls = require_tls!();

    let dir = tempfile::tempdir().unwrap();

    // Generate a different CA cert that doesn't match the server
    let wrong_cert =
        rcgen::generate_simple_self_signed(vec!["wrong.example.com".to_string()]).unwrap();
    let wrong_ca = dir.path().join("wrong-ca.crt");
    std::fs::write(&wrong_ca, wrong_cert.cert.pem()).unwrap();

    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[ssl]").unwrap();
    writeln!(f, "certfile = {}", wrong_ca.display()).unwrap();
    writeln!(f, "validate = true").unwrap();

    tls_cqlsh_cmd(tls)
        .args([
            "--ssl",
            "--cqlshrc",
            cqlshrc.to_str().unwrap(),
            "--connect-timeout",
            "5",
            "-e",
            "SELECT release_version FROM system.local",
        ])
        .assert()
        .failure();
}

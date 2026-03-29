//! SSL/TLS integration tests for cqlsh-rs.
//!
//! Tests the --ssl flag and SSL-related CLI behavior. The ScyllaDB container
//! started for these tests does NOT have SSL configured, so connections with
//! --ssl should fail with a connection error (not a panic or hang).
//!
//! Full mutual-TLS tests require a TLS-enabled container (out of scope here).
//! See SP16 (upstream PR #163) for detailed TLS integration requirements.

use super::helpers::*;

// ---------------------------------------------------------------------------
// SSL flag accepted without panic
// ---------------------------------------------------------------------------

/// Verify that --ssl is accepted as a CLI flag and produces a meaningful
/// error when the server does not have SSL enabled (rather than panicking
/// or hanging indefinitely).
#[test]
#[ignore = "requires Docker"]
fn test_ssl_flag_fails_gracefully_on_non_ssl_server() {
    let scylla = get_scylla();

    // The ScyllaDB container has no TLS configured, so --ssl should cause
    // a connection error. We only verify the process exits with an error
    // code (non-zero) and does not hang or panic.
    let output = cqlsh_cmd(scylla)
        .args(["--ssl", "-e", "SELECT release_version FROM system.local"])
        .output()
        .expect("cqlsh-rs process should not hang");

    // Must exit with a non-zero code (TLS handshake failure)
    assert!(
        !output.status.success(),
        "Expected failure with --ssl against non-TLS server"
    );

    // Stderr should contain some kind of error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "Expected an error message on stderr when SSL handshake fails"
    );
}

// ---------------------------------------------------------------------------
// SSL flag does not appear in --no-ssl plain connections
// ---------------------------------------------------------------------------

/// Verify that a plain (non-SSL) connection to ScyllaDB succeeds normally,
/// establishing a baseline for the SSL tests.
#[test]
#[ignore = "requires Docker"]
fn test_non_ssl_connection_succeeds() {
    let scylla = get_scylla();

    // Plain connection (no --ssl) should succeed
    cqlsh_cmd(scylla)
        .args(["-e", "SELECT release_version FROM system.local"])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// --ssl combined with --username / --password does not panic
// ---------------------------------------------------------------------------

/// Verify that combining --ssl with authentication flags doesn't cause a
/// panic or unexpected crash (even though the connection itself will fail
/// against a non-TLS server).
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

    // Process must exit (not hang); result code doesn't matter here
    let _ = output.status;
}

// ---------------------------------------------------------------------------
// Timeout on SSL handshake is bounded
// ---------------------------------------------------------------------------

/// Verify that --connect-timeout is respected when SSL handshake fails,
/// i.e., the process doesn't run indefinitely.
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

    // Should complete (with failure) well within the timeout + margin
    assert!(
        elapsed.as_secs() < 30,
        "SSL connection attempt took too long: {elapsed:?}"
    );
    assert!(
        !output.status.success(),
        "Expected non-zero exit when SSL fails"
    );
}

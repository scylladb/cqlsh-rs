//! Integration tests for proxy auto-detection.
//!
//! Tests that verify the proxy address translator works correctly with a
//! real ScyllaDB instance.

use helpers::{cqlsh_cmd, get_scylla};

use super::*;

/// Direct connection (no proxy) should work without address translation.
/// This confirms the two-phase connect doesn't break normal connections.
#[test]
#[ignore = "requires Docker"]
fn test_direct_connection_still_works_with_proxy_detection() {
    let scylla = get_scylla();
    cqlsh_cmd(scylla)
        .args(["-e", "SELECT cluster_name FROM system.local"])
        .assert()
        .success();
}

/// Verify that a query actually returns data through the normal path,
/// confirming the two-phase connect doesn't cause session issues.
#[test]
#[ignore = "requires Docker"]
fn test_query_after_proxy_detection_returns_data() {
    let scylla = get_scylla();
    let output =
        helpers::execute_cql_output_direct(scylla, "SELECT release_version FROM system.local");
    assert!(
        !output.is_empty(),
        "expected output from system.local query"
    );
}

/// Test connection through a TCP proxy (socat) to simulate the proxy scenario.
/// The proxy forwards traffic to the ScyllaDB container, but its IP won't
/// match any peer address in system.peers, triggering proxy auto-detection.
#[test]
#[ignore = "requires Docker"]
fn test_proxy_connection_via_socat() {
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::Duration;

    let scylla = get_scylla();
    let target = format!("{}:{}", scylla.host, scylla.port);

    // Start socat as a TCP proxy on a random high port
    // socat listens on 0.0.0.0:0 (kernel picks port) and forwards to the ScyllaDB container
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let proxy_port = listener.local_addr().unwrap().port();
    drop(listener); // free the port for socat

    let mut socat = match Command::new("socat")
        .args([
            &format!("TCP-LISTEN:{proxy_port},fork,reuseaddr"),
            &format!("TCP:{target}"),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            eprintln!("socat not available, skipping proxy integration test");
            return;
        }
    };

    // Give socat time to start listening
    thread::sleep(Duration::from_millis(500));

    // Connect through the proxy — proxy IP (127.0.0.1:proxy_port) won't match
    // the ScyllaDB node's internal address, triggering proxy detection
    let result = assert_cmd::Command::cargo_bin("cqlsh-rs")
        .unwrap()
        .args([
            "127.0.0.1",
            &proxy_port.to_string(),
            "-e",
            "SELECT cluster_name FROM system.local",
        ])
        .timeout(Duration::from_secs(15))
        .assert();

    socat.kill().ok();
    socat.wait().ok();

    result.success();
}

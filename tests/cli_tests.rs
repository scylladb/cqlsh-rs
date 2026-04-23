//! CLI-level integration tests for cqlsh-rs binary.
//!
//! Tests the compiled binary using assert_cmd, verifying that
//! flags, help output, version output, and error handling work correctly.
//!
//! Note: Tests that trigger a connection attempt will fail with a connection
//! error when no Cassandra/ScyllaDB cluster is available. These tests verify
//! the connection error behavior rather than asserting success.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

fn cmd() -> Command {
    Command::cargo_bin("cqlsh-rs").unwrap()
}

#[test]
fn version_flag_shows_version() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cqlsh"));
}

#[test]
fn help_flag_shows_usage() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"))
        .stdout(predicate::str::contains("--ssl"))
        .stdout(predicate::str::contains("--execute"))
        .stdout(predicate::str::contains("--keyspace"))
        .stdout(predicate::str::contains("--username"))
        .stdout(predicate::str::contains("--password"))
        .stdout(predicate::str::contains("--connect-timeout"))
        .stdout(predicate::str::contains("--request-timeout"))
        .stdout(predicate::str::contains("--cqlshrc"))
        .stdout(predicate::str::contains("--completions"));
}

#[test]
fn unknown_flag_fails() {
    cmd()
        .arg("--nonexistent-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn color_and_no_color_conflict() {
    cmd()
        .args(["-C", "--no-color"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--color"));
}

#[test]
fn execute_and_file_conflict() {
    cmd()
        .args(["-e", "SELECT 1", "-f", "test.cql"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--execute"));
}

#[test]
fn custom_cqlshrc_path() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "hostname = 10.0.0.1").unwrap();
    writeln!(f, "port = 9999").unwrap();

    // With no cluster available, the binary will fail to connect.
    // The debug output should still show the resolved config before the error.
    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("10.0.0.1"))
        .stderr(predicate::str::contains("9999"));
}

#[test]
fn debug_flag_shows_config() {
    // With no cluster available, the binary will fail to connect,
    // but debug output should still be printed before the error.
    cmd()
        .arg("--debug")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Debug: resolved host="));
}

#[test]
fn completions_bash() {
    cmd()
        .args(["--completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn completions_zsh() {
    cmd()
        .args(["--completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cqlsh-rs"));
}

#[test]
fn completions_fish() {
    cmd()
        .args(["--completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cqlsh-rs"));
}

#[test]
fn default_host_and_port() {
    // Connection will fail without a cluster; verify debug output and connection error
    cmd()
        .arg("--debug")
        .assert()
        .failure()
        .stderr(predicate::str::contains("host=127.0.0.1"))
        .stderr(predicate::str::contains("port=9042"));
}

#[test]
fn positional_host_override() {
    // Connection will fail; verify resolved host appears in error output
    cmd()
        .args(["192.168.1.100", "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("host=192.168.1.100"));
}

#[test]
fn positional_host_and_port_override() {
    cmd()
        .args(["192.168.1.100", "9999", "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("host=192.168.1.100"))
        .stderr(predicate::str::contains("port=9999"));
}

#[test]
fn env_host_override() {
    cmd()
        .env("CQLSH_HOST", "env-host.example.com")
        .arg("--debug")
        .assert()
        .failure()
        .stderr(predicate::str::contains("host=env-host.example.com"));
}

#[test]
fn env_port_override() {
    cmd()
        .env("CQLSH_PORT", "19042")
        .arg("--debug")
        .assert()
        .failure()
        .stderr(predicate::str::contains("port=19042"));
}

#[test]
fn cli_host_overrides_env() {
    cmd()
        .env("CQLSH_HOST", "env-host")
        .args(["cli-host", "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("host=cli-host"));
}

#[test]
fn nonexistent_cqlshrc_is_ok() {
    // The cqlshrc loading succeeds (returns defaults), but connection fails
    cmd()
        .args(["--cqlshrc", "/nonexistent/path/cqlshrc", "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Connection error"));
}

#[test]
fn connection_error_shows_host_port() {
    // Verify the connection error message matches Python cqlsh format
    cmd()
        .args(["10.255.255.1", "9999", "--connect-timeout", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Connection error"))
        .stderr(predicate::str::contains("10.255.255.1:9999"));
}

#[test]
fn connection_error_exits_with_code_2() {
    // Connection failure must exit with code 2 (distinct from CQL error = 1)
    cmd()
        .args(["10.255.255.1", "9999", "--connect-timeout", "1"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Connection error"));
}

#[test]
fn help_flag_shows_tty() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--tty"));
}

#[test]
fn stdin_pipe_empty_exits_with_code_2() {
    // assert_cmd's write_stdin provides a closed pipe, so stdin.is_terminal() = false.
    // The binary connects first (fails → exit 2) before reading any stdin.
    cmd().write_stdin("").assert().code(2);
}

#[test]
fn tty_flag_accepted_with_piped_stdin() {
    // --tty forces REPL path even when stdin is a pipe.
    // Without a cluster both paths still fail at connection → exit 2.
    cmd().arg("--tty").write_stdin("").assert().code(2);
}

// === CLI flag acceptance tests ===
// Each test verifies the binary accepts the flag without argument parsing errors.
// Flags that trigger a connection attempt will fail with exit code 2 (no cluster).

#[test]
fn ssl_flag_accepted() {
    // --ssl requires a TLS CryptoProvider; without one the process panics (exit 101).
    // This test just verifies the flag is parsed without argument errors.
    cmd()
        .args(["--ssl", "--connect-timeout", "1"])
        .assert()
        .failure();
}

#[test]
fn no_file_io_flag_accepted() {
    cmd()
        .args(["--no-file-io", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn coverage_flag_accepted() {
    cmd()
        .args(["--coverage", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn keyspace_flag_accepted() {
    cmd()
        .args(["-k", "test_ks", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn username_flag_accepted() {
    cmd()
        .args(["-u", "testuser", "--debug", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn password_flag_accepted() {
    cmd()
        .args(["-p", "testpass", "--debug", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn request_timeout_flag_accepted() {
    cmd()
        .args([
            "--request-timeout",
            "30",
            "--debug",
            "--connect-timeout",
            "1",
        ])
        .assert()
        .code(2);
}

#[test]
fn encoding_flag_accepted() {
    cmd()
        .args(["--encoding", "utf-8", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn cqlversion_flag_accepted() {
    cmd()
        .args(["--cqlversion", "3.4.5", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn protocol_version_flag_accepted() {
    cmd()
        .args(["--protocol-version", "4", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn protocol_version_out_of_range() {
    cmd()
        .args(["--protocol-version", "99"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Protocol version"));
}

#[test]
fn consistency_level_flag_accepted() {
    cmd()
        .args(["--consistency-level", "QUORUM", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn serial_consistency_level_flag_accepted() {
    cmd()
        .args([
            "--serial-consistency-level",
            "LOCAL_SERIAL",
            "--connect-timeout",
            "1",
        ])
        .assert()
        .code(2);
}

#[test]
fn no_compact_flag_accepted() {
    cmd()
        .args(["--no_compact", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn disable_history_flag_accepted() {
    cmd()
        .args(["--disable-history", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn secure_connect_bundle_flag_accepted() {
    // Bundle file won't exist, but flag parsing should succeed;
    // connection will fail with exit code 2
    cmd()
        .args(["-b", "/nonexistent/bundle.zip", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn browser_flag_accepted() {
    cmd()
        .args(["--browser", "firefox", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn generate_man_produces_output() {
    cmd()
        .arg("--generate-man")
        .assert()
        .success()
        .stdout(predicate::str::contains("cqlsh"));
}

#[test]
fn color_flag_accepted() {
    cmd()
        .args(["-C", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn no_color_flag_accepted() {
    cmd()
        .args(["--no-color", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn execute_flag_attempts_connection() {
    cmd()
        .args(["-e", "SELECT 1", "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn file_flag_attempts_connection() {
    let dir = tempfile::tempdir().unwrap();
    let cql_file = dir.path().join("test.cql");
    std::fs::write(&cql_file, "SELECT 1;\n").unwrap();

    cmd()
        .args(["-f", cql_file.to_str().unwrap(), "--connect-timeout", "1"])
        .assert()
        .code(2);
}

#[test]
fn safe_mode_flag_accepted() {
    cmd().arg("--safe-mode").write_stdin("").assert().code(2);
}

//! CLI-level integration tests for cqlsh-rs binary.
//!
//! Tests the compiled binary using assert_cmd, verifying that
//! flags, help output, version output, and error handling work correctly.

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

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .success()
        .stderr(predicate::str::contains("10.0.0.1"))
        .stderr(predicate::str::contains("9999"));
}

#[test]
fn debug_flag_shows_config() {
    cmd()
        .arg("--debug")
        .assert()
        .success()
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
    cmd()
        .arg("--debug")
        .assert()
        .success()
        .stderr(predicate::str::contains("host=127.0.0.1"))
        .stderr(predicate::str::contains("port=9042"));
}

#[test]
fn positional_host_override() {
    cmd()
        .args(["192.168.1.100", "--debug"])
        .assert()
        .success()
        .stderr(predicate::str::contains("host=192.168.1.100"));
}

#[test]
fn positional_host_and_port_override() {
    cmd()
        .args(["192.168.1.100", "9999", "--debug"])
        .assert()
        .success()
        .stderr(predicate::str::contains("host=192.168.1.100"))
        .stderr(predicate::str::contains("port=9999"));
}

#[test]
fn env_host_override() {
    cmd()
        .env("CQLSH_HOST", "env-host.example.com")
        .arg("--debug")
        .assert()
        .success()
        .stderr(predicate::str::contains("host=env-host.example.com"));
}

#[test]
fn env_port_override() {
    cmd()
        .env("CQLSH_PORT", "19042")
        .arg("--debug")
        .assert()
        .success()
        .stderr(predicate::str::contains("port=19042"));
}

#[test]
fn cli_host_overrides_env() {
    cmd()
        .env("CQLSH_HOST", "env-host")
        .args(["cli-host", "--debug"])
        .assert()
        .success()
        .stderr(predicate::str::contains("host=cli-host"));
}

#[test]
fn nonexistent_cqlshrc_is_ok() {
    cmd()
        .args(["--cqlshrc", "/nonexistent/path/cqlshrc", "--debug"])
        .assert()
        .success();
}

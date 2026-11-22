use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

fn cmd() -> Command {
    Command::cargo_bin("cqlsh-rs").unwrap()
}

#[test]
fn connection_hostname_from_cqlshrc() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "hostname = 10.0.0.1").unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("host=10.0.0.1"));
}

#[test]
fn connection_port_from_cqlshrc() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "port = 9999").unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("port=9999"));
}

#[test]
fn connection_hostname_and_port() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "hostname = 10.0.0.2").unwrap();
    writeln!(f, "port = 8888").unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("host=10.0.0.2"))
        .stderr(predicate::str::contains("port=8888"));
}

#[test]
fn connect_timeout_from_cqlshrc() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "connect_timeout = 30").unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Debug: resolved host="));
}

#[test]
fn request_timeout_from_cqlshrc() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "request_timeout = 60").unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Debug: resolved host="));
}

#[test]
fn authentication_username_from_cqlshrc() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[authentication]").unwrap();
    writeln!(f, "username = testuser").unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Debug: resolved host="));
}

#[test]
fn ui_encoding_from_cqlshrc() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[ui]").unwrap();
    writeln!(f, "encoding = latin-1").unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Using 'latin-1' encoding"));
}

#[test]
fn cql_version_from_cqlshrc() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[cql]").unwrap();
    writeln!(f, "version = 3.4.5").unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cqlversion=3.4.5"));
}

#[test]
fn ssl_section_certfile() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[ssl]").unwrap();
    writeln!(f, "certfile = /tmp/fake.pem").unwrap();
    writeln!(f, "validate = true").unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Debug: resolved host="));
}

#[test]
fn empty_cqlshrc_uses_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    std::fs::File::create(&cqlshrc).unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("host=127.0.0.1"))
        .stderr(predicate::str::contains("port=9042"));
}

#[test]
fn cli_overrides_cqlshrc_host() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "hostname = 10.0.0.1").unwrap();

    cmd()
        .args([
            "--cqlshrc",
            cqlshrc.to_str().unwrap(),
            "--debug",
            "192.168.1.1",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("host=192.168.1.1"));
}

#[test]
fn cli_overrides_cqlshrc_port() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "port = 9999").unwrap();

    cmd()
        .args([
            "--cqlshrc",
            cqlshrc.to_str().unwrap(),
            "--debug",
            "127.0.0.1",
            "7777",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("port=7777"));
}

#[test]
fn env_var_overrides_cqlshrc_hostname() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "hostname = 10.0.0.1").unwrap();

    cmd()
        .env("CQLSH_HOST", "10.10.10.10")
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("host=10.10.10.10"));
}

#[test]
fn full_cqlshrc_all_sections() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[authentication]").unwrap();
    writeln!(f, "username = cassandra").unwrap();
    writeln!(f, "password = cassandra").unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "hostname = 127.0.0.1").unwrap();
    writeln!(f, "port = 9042").unwrap();
    writeln!(f, "connect_timeout = 5").unwrap();
    writeln!(f, "request_timeout = 10").unwrap();
    writeln!(f, "[ssl]").unwrap();
    writeln!(f, "certfile = /tmp/ca.pem").unwrap();
    writeln!(f, "validate = false").unwrap();
    writeln!(f, "[ui]").unwrap();
    writeln!(f, "color = on").unwrap();
    writeln!(f, "encoding = utf-8").unwrap();
    writeln!(f, "[cql]").unwrap();
    writeln!(f, "version = 3.4.0").unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--debug"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Debug: resolved host="));
}

// --- [client_routes] ---

#[test]
fn client_routes_from_cqlshrc() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "hostname = 10.255.255.1").unwrap();
    writeln!(f, "[client_routes]").unwrap();
    writeln!(f, "proxies = conn-a=proxy-a.example.com,conn-b").unwrap();
    writeln!(f, "advanced_shard_awareness = true").unwrap();

    cmd()
        .args([
            "--cqlshrc",
            cqlshrc.to_str().unwrap(),
            "--debug",
            "--connect-timeout",
            "1",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Using client routes: true"))
        .stderr(predicate::str::contains(
            "Debug: client routes=[conn-a=proxy-a.example.com, conn-b]",
        ))
        .stderr(predicate::str::contains(
            "Debug: client routes advanced_shard_awareness=true",
        ));
}

#[test]
fn client_routes_multiline_proxies_from_cqlshrc() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "hostname = 10.255.255.1").unwrap();
    writeln!(f, "[client_routes]").unwrap();
    writeln!(f, "proxies = conn-a=proxy-a.example.com,").unwrap();
    writeln!(f, "          conn-b").unwrap();

    cmd()
        .args([
            "--cqlshrc",
            cqlshrc.to_str().unwrap(),
            "--debug",
            "--connect-timeout",
            "1",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "Debug: client routes=[conn-a=proxy-a.example.com, conn-b]",
        ));
}

#[test]
fn cli_client_route_overrides_cqlshrc() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "hostname = 10.255.255.1").unwrap();
    writeln!(f, "[client_routes]").unwrap();
    writeln!(f, "proxies = file-conn").unwrap();

    cmd()
        .args([
            "--cqlshrc",
            cqlshrc.to_str().unwrap(),
            "--client-route",
            "cli-conn",
            "--debug",
            "--connect-timeout",
            "1",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Debug: client routes=[cli-conn]"))
        .stderr(predicate::str::contains("file-conn").not());
}

#[test]
fn no_client_routes_section_disables_them() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[connection]").unwrap();
    writeln!(f, "hostname = 10.255.255.1").unwrap();

    cmd()
        .args([
            "--cqlshrc",
            cqlshrc.to_str().unwrap(),
            "--debug",
            "--connect-timeout",
            "1",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Using client routes: false"));
}

#[test]
fn cqlshrc_client_routes_conflict_with_ssl() {
    let dir = tempfile::tempdir().unwrap();
    let cqlshrc = dir.path().join("cqlshrc");
    let mut f = std::fs::File::create(&cqlshrc).unwrap();
    writeln!(f, "[client_routes]").unwrap();
    writeln!(f, "proxies = conn-a").unwrap();

    cmd()
        .args(["--cqlshrc", cqlshrc.to_str().unwrap(), "--ssl"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Client routes do not support SSL/TLS",
        ));
}

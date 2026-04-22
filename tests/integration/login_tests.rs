//! Integration tests for LOGIN command in non-interactive mode.

use predicates::prelude::*;

use super::helpers::*;

#[test]
#[ignore = "requires Docker"]
fn login_bare_shows_usage_on_stderr() {
    let scylla = get_scylla();
    cqlsh_cmd(scylla)
        .args(["-e", "LOGIN"])
        .assert()
        .stderr(predicate::str::contains("Usage: LOGIN"))
        .code(1);
}

#[test]
#[ignore = "requires Docker"]
fn login_with_credentials_reconnects_and_queries() {
    let scylla = get_scylla();
    cqlsh_cmd(scylla)
        .args([
            "-e",
            "LOGIN cassandra 'cassandra'; SELECT key FROM system.local",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("local"));
}

#[test]
#[ignore = "requires Docker"]
fn login_username_only_reconnects() {
    let scylla = get_scylla();
    cqlsh_cmd(scylla)
        .args(["-e", "LOGIN cassandra; SELECT key FROM system.local"])
        .assert()
        .success()
        .stdout(predicate::str::contains("local"));
}

#[test]
#[ignore = "requires Docker"]
fn login_via_stdin_reconnects_and_runs_next_statement() {
    let scylla = get_scylla();
    cqlsh_cmd(scylla)
        .write_stdin("LOGIN cassandra 'cassandra';\nSELECT key FROM system.local;\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("local"));
}

#[test]
#[ignore = "requires Docker"]
fn login_preserves_keyspace_context() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "login_ks");

    execute_cql(scylla, &format!("CREATE TABLE {ks}.t (id int PRIMARY KEY)")).success();

    cqlsh_cmd(scylla)
        .args([
            "-k",
            &ks,
            "-e",
            "LOGIN cassandra 'cassandra'; SELECT id FROM t",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("id"));

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn login_multiple_times_in_sequence() {
    let scylla = get_scylla();
    cqlsh_cmd(scylla)
        .args([
            "-e",
            "LOGIN cassandra 'cassandra'; LOGIN cassandra 'cassandra'; SELECT key FROM system.local",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("local"));
}

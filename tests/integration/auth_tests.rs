use predicates::prelude::*;

use super::helpers::*;

/// Command with SSL configured (if external) but NO credentials added.
fn ssl_cmd(scylla: &ScyllaContainer) -> assert_cmd::Command {
    let mut cmd = cqlsh_cmd(scylla);
    if let Ok(ca) = std::env::var("CQLSH_TEST_SSL_CA_PATH") {
        let dir = tempfile::tempdir().unwrap();
        let cqlshrc = dir.path().join("cqlshrc");
        std::fs::write(&cqlshrc, format!("[ssl]\ncertfile = {ca}\n")).unwrap();
        cmd.args(["--ssl", "--cqlshrc", cqlshrc.to_str().unwrap()]);
        std::mem::forget(dir);
    }
    cmd
}

/// Command with SSL + admin credentials (for setup statements).
fn admin_cmd(scylla: &ScyllaContainer) -> assert_cmd::Command {
    let mut cmd = ssl_cmd(scylla);
    if let (Ok(user), Ok(pass)) = (
        std::env::var("CQLSH_TEST_USERNAME"),
        std::env::var("CQLSH_TEST_PASSWORD"),
    ) {
        cmd.args(["-u", &user, "-p", &pass]);
    }
    cmd
}

fn admin_execute_cql(scylla: &ScyllaContainer, cql: &str) -> assert_cmd::assert::Assert {
    let stmt = if cql.trim_end().ends_with(';') {
        cql.to_string()
    } else {
        format!("{};", cql.trim_end())
    };
    admin_cmd(scylla).args(["-e", &stmt]).assert()
}

#[test]
#[ignore = "requires Docker"]
fn create_role_and_connect_as_non_admin() {
    let scylla = get_scylla();

    admin_execute_cql(
        scylla,
        "CREATE ROLE IF NOT EXISTS testuser WITH PASSWORD = 'testpass' AND LOGIN = true",
    )
    .success();

    admin_execute_cql(scylla, "GRANT SELECT ON KEYSPACE system TO testuser").success();

    ssl_cmd(scylla)
        .args([
            "-u",
            "testuser",
            "-p",
            "testpass",
            "-e",
            "SELECT key FROM system.local",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("local"));
}

#[test]
#[ignore = "requires Docker"]
fn login_as_non_admin_user() {
    let scylla = get_scylla();

    admin_execute_cql(
        scylla,
        "CREATE ROLE IF NOT EXISTS loginuser WITH PASSWORD = 'loginpass' AND LOGIN = true",
    )
    .success();

    admin_execute_cql(scylla, "GRANT SELECT ON KEYSPACE system TO loginuser").success();

    admin_cmd(scylla)
        .args([
            "-e",
            "LOGIN loginuser 'loginpass'; SELECT key FROM system.local",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("local"));
}

#[test]
#[ignore = "requires Docker"]
fn non_admin_denied_without_grant() {
    let scylla = get_scylla();

    admin_execute_cql(
        scylla,
        "CREATE ROLE IF NOT EXISTS noperm WITH PASSWORD = 'noperm' AND LOGIN = true",
    )
    .success();

    ssl_cmd(scylla)
        .args(["-u", "noperm", "-p", "noperm", "-e", "CREATE KEYSPACE denied_ks WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1}"])
        .assert()
        .failure();
}

#[test]
#[ignore = "requires Docker"]
fn wrong_password_rejected() {
    let scylla = get_scylla();

    ssl_cmd(scylla)
        .args([
            "-u",
            "cassandra",
            "-p",
            "wrongpassword",
            "-e",
            "SELECT key FROM system.local",
        ])
        .assert()
        .failure();
}

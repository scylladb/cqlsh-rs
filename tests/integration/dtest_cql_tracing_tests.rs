//! Integration tests ported from scylla-dtest cql_tracing_test.py.
//!
//! Single-node adaptation of `TestCqlTracing::test_tracing_simple`: the dtest
//! runs on 3 nodes with RF=3 and `CONSISTENCY ALL`, which a single node cannot
//! satisfy (system_traces has RF=2), so the CONSISTENCY ALL step is dropped and
//! the per-node source IP checks are omitted.

use super::helpers::*;

/// Run `-e <cql>` with `--no-color` and return stdout, asserting success.
fn run(scylla: &ScyllaContainer, cql: &str) -> String {
    let output = cqlsh_cmd(scylla)
        .args(["--no-color", "-e", cql])
        .output()
        .expect("failed to execute cqlsh-rs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "cqlsh-rs failed for {cql:?}: {}\n{stdout}",
        String::from_utf8_lossy(&output.stderr)
    );
    stdout
}

fn assert_traced(out: &str) {
    assert!(
        out.contains("Tracing session: "),
        "query was run without tracing: {out}"
    );
    assert!(
        out.contains("Request complete "),
        "Expected substring \"Request complete\" was not found: {out}"
    );
}

#[test]
#[ignore = "requires Docker"]
fn test_tracing_simple() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "tracing");

    execute_cql(
        scylla,
        &format!(
            "CREATE TABLE {ks}.users (userid uuid PRIMARY KEY, firstname text, lastname text, age int)"
        ),
    )
    .success();

    let out = run(scylla, "TRACING ON");
    assert!(
        out.contains("Tracing is enabled"),
        "Tracing has not been enabled: {out}"
    );

    let out = run(scylla, &format!("TRACING ON; SELECT * from {ks}.users"));
    assert_traced(&out);

    let out = run(
        scylla,
        &format!(
            "TRACING ON; INSERT INTO {ks}.users (userid, firstname, lastname, age) \
             VALUES (550e8400-e29b-41d4-a716-446655440000, 'Frodo', 'Baggins', 32)"
        ),
    );
    assert_traced(&out);

    let out = run(
        scylla,
        &format!(
            "TRACING ON; SELECT firstname, lastname FROM {ks}.users \
             WHERE userid = 550e8400-e29b-41d4-a716-446655440000"
        ),
    );
    assert_traced(&out);
    assert!(
        out.contains("Frodo") && out.contains("Baggins"),
        "Expected the Frodo/Baggins row: {out}"
    );

    drop_test_keyspace(scylla, &ks);
}

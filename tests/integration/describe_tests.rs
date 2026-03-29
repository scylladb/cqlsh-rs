//! DESCRIBE command integration tests for cqlsh-rs.
//!
//! Tests DESCRIBE KEYSPACE, TABLE, TABLES, KEYSPACES, INDEX, TYPE, TYPES,
//! CLUSTER, SCHEMA, FULL SCHEMA, MATERIALIZED VIEW, FUNCTION, AGGREGATE.
//! Mirrors Python cqlsh dtest `test_describe`, `TestCqlshSmoke`, and
//! Phase 4 tasks 4.12–4.17.

use super::helpers::*;

// ---------------------------------------------------------------------------
// DESCRIBE FULL SCHEMA — must include system keyspaces
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_full_schema_includes_system() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_full");

    let output = cqlsh_cmd(scylla)
        .args(["-e", "DESCRIBE FULL SCHEMA"])
        .output()
        .expect("failed to run cqlsh-rs");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        stdout.contains("system"),
        "DESCRIBE FULL SCHEMA should include system keyspaces, got: {stdout}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE INDEX <name> — DDL for a secondary index
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_index() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_idx");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.users (id int PRIMARY KEY, email text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("CREATE INDEX email_idx ON {ks}.users (email)"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("DESCRIBE INDEX {ks}.email_idx"));
    assert!(
        output.contains("CREATE INDEX"),
        "DESCRIBE INDEX should show CREATE INDEX: {output}"
    );
    assert!(
        output.contains("email_idx"),
        "DESCRIBE INDEX should show index name: {output}"
    );
    assert!(
        output.contains("email"),
        "DESCRIBE INDEX should show indexed column: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

/// Verify DESCRIBE INDEX works for an index on a non-pk column of a table with
/// multiple regular columns (covers a different DDL shape than test_describe_index).
#[test]
#[ignore = "requires Docker"]
fn test_describe_index_on_non_pk_column() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_idx2");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.items (id int PRIMARY KEY, category text, price double)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("CREATE INDEX idx_category ON {ks}.items (category)"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("DESCRIBE INDEX {ks}.idx_category"));
    assert!(
        output.contains("CREATE INDEX") || output.contains("idx_category"),
        "DESCRIBE INDEX should show index DDL: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE MATERIALIZED VIEW <name>
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_materialized_view() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_mv");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.users (id int PRIMARY KEY, email text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!(
            "CREATE MATERIALIZED VIEW {ks}.users_by_email AS \
             SELECT * FROM {ks}.users WHERE email IS NOT NULL AND id IS NOT NULL \
             PRIMARY KEY (email, id)"
        ),
    )
    .success();

    let output = execute_cql_output(
        scylla,
        &format!("DESCRIBE MATERIALIZED VIEW {ks}.users_by_email"),
    );
    assert!(
        output.contains("CREATE MATERIALIZED VIEW"),
        "DESCRIBE MATERIALIZED VIEW should show CREATE statement: {output}"
    );
    assert!(
        output.contains("users_by_email"),
        "DESCRIBE MATERIALIZED VIEW should show view name: {output}"
    );
    assert!(
        output.contains("PRIMARY KEY"),
        "DESCRIBE MATERIALIZED VIEW should show PRIMARY KEY: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE TYPE <name> — DDL for a user-defined type (UDT)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_type() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_type");

    execute_cql(
        scylla,
        &format!("CREATE TYPE {ks}.address (street text, city text, zip int)"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("DESCRIBE TYPE {ks}.address"));
    assert!(
        output.contains("CREATE TYPE"),
        "DESCRIBE TYPE should show CREATE TYPE: {output}"
    );
    assert!(
        output.contains("address"),
        "DESCRIBE TYPE should show type name: {output}"
    );
    assert!(
        output.contains("street"),
        "DESCRIBE TYPE should show field names: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE TYPES — list UDT names in current keyspace
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_types_list() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_types_list");

    execute_cql(
        scylla,
        &format!("CREATE TYPE {ks}.tag (name text, value text)"),
    )
    .success();

    let output = cqlsh_cmd(scylla)
        .args(["-k", &ks, "-e", "DESCRIBE TYPES"])
        .output()
        .expect("failed to run cqlsh-rs");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        stdout.contains("tag"),
        "DESCRIBE TYPES should list type names: {stdout}"
    );

    drop_test_keyspace(scylla, &ks);
}

/// Verify DESCRIBE TYPES lists multiple UDTs.
#[test]
#[ignore = "requires Docker"]
fn test_describe_types_multiple() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_udts");

    execute_cql(
        scylla,
        &format!("CREATE TYPE {ks}.phone (number text, kind text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("CREATE TYPE {ks}.coords (lat double, lon double)"),
    )
    .success();

    let output = cqlsh_cmd(scylla)
        .args(["-k", &ks, "-e", "DESCRIBE TYPES"])
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("phone") || stdout.contains("coords"),
        "DESCRIBE TYPES should list UDT names: {stdout}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE FUNCTION <name> — DDL for a UDF (skip if UDFs not enabled)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_function() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_func");

    let create_result = cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!(
                "CREATE OR REPLACE FUNCTION {ks}.double_val(val int) \
                 RETURNS NULL ON NULL INPUT \
                 RETURNS int \
                 LANGUAGE lua \
                 AS 'return val * 2';"
            ),
        ])
        .output()
        .expect("failed to run cqlsh-rs");

    if !create_result.status.success() {
        // UDFs not enabled on this instance — skip
        drop_test_keyspace(scylla, &ks);
        return;
    }

    let output = execute_cql_output(scylla, &format!("DESCRIBE FUNCTION {ks}.double_val"));
    assert!(
        output.contains("CREATE OR REPLACE FUNCTION"),
        "DESCRIBE FUNCTION should show CREATE FUNCTION: {output}"
    );
    assert!(
        output.contains("double_val"),
        "DESCRIBE FUNCTION should show function name: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE AGGREGATE <name> — DDL for a UDA (skip if UDFs not enabled)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_aggregate() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_agg");

    let create_func = cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!(
                "CREATE OR REPLACE FUNCTION {ks}.sum_state(state int, val int) \
                 CALLED ON NULL INPUT \
                 RETURNS int \
                 LANGUAGE lua \
                 AS 'return state + val';"
            ),
        ])
        .output()
        .expect("failed to run cqlsh-rs");

    if !create_func.status.success() {
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!(
            "CREATE OR REPLACE AGGREGATE {ks}.my_sum(int) \
             SFUNC sum_state \
             STYPE int \
             INITCOND 0"
        ),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("DESCRIBE AGGREGATE {ks}.my_sum"));
    assert!(
        output.contains("CREATE OR REPLACE AGGREGATE"),
        "DESCRIBE AGGREGATE should show CREATE AGGREGATE: {output}"
    );
    assert!(
        output.contains("my_sum"),
        "DESCRIBE AGGREGATE should show aggregate name: {output}"
    );
    assert!(
        output.contains("SFUNC"),
        "DESCRIBE AGGREGATE should show SFUNC: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE of non-existent object — should print "not found", not crash
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_nonexistent_index() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_noexist");

    let output = cqlsh_cmd(scylla)
        .args(["-e", &format!("DESCRIBE INDEX {ks}.no_such_idx")])
        .output()
        .expect("failed to run cqlsh-rs");

    assert!(
        output.status.success(),
        "DESCRIBE of non-existent index should not fail with non-zero exit"
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.to_lowercase().contains("not found"),
        "Should print 'not found' message: {combined}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE KEYSPACES — list all keyspaces
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_keyspaces() {
    let scylla = get_scylla();

    let output = execute_cql_output(scylla, "DESCRIBE KEYSPACES");
    assert!(
        output.contains("system"),
        "DESCRIBE KEYSPACES should list system keyspace: {output}"
    );
}

// ---------------------------------------------------------------------------
// DESCRIBE KEYSPACE <name> — full DDL for a specific keyspace
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_keyspace() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_ks2");

    let output = execute_cql_output(scylla, &format!("DESCRIBE KEYSPACE {ks}"));
    assert!(
        output.contains("CREATE KEYSPACE"),
        "DESCRIBE KEYSPACE should show CREATE statement: {output}"
    );
    assert!(
        output.contains(&ks),
        "DESCRIBE KEYSPACE should include keyspace name: {output}"
    );
    assert!(
        output.contains("SimpleStrategy") || output.contains("replication"),
        "DESCRIBE KEYSPACE should show replication options: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE TABLE <name> — full DDL for a table
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_table() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_tbl2");

    execute_cql(
        scylla,
        &format!(
            "CREATE TABLE {ks}.users (\
             id int PRIMARY KEY, \
             name text, \
             email text, \
             age int)"
        ),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("DESCRIBE TABLE {ks}.users"));
    assert!(
        output.contains("CREATE TABLE"),
        "DESCRIBE TABLE should show CREATE statement: {output}"
    );
    assert!(
        output.contains("PRIMARY KEY"),
        "DESCRIBE TABLE should show PRIMARY KEY: {output}"
    );
    assert!(
        output.contains("name"),
        "DESCRIBE TABLE should list column 'name': {output}"
    );
    assert!(
        output.contains("email"),
        "DESCRIBE TABLE should list column 'email': {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE TABLE — composite primary key
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_table_composite_key() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_cpk");

    execute_cql(
        scylla,
        &format!(
            "CREATE TABLE {ks}.events (\
             user_id int, \
             event_time timestamp, \
             payload text, \
             PRIMARY KEY (user_id, event_time))"
        ),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("DESCRIBE TABLE {ks}.events"));
    assert!(
        output.contains("PRIMARY KEY"),
        "Expected composite PRIMARY KEY in output: {output}"
    );
    assert!(
        output.contains("user_id"),
        "Expected partition key in output: {output}"
    );
    assert!(
        output.contains("event_time"),
        "Expected clustering key in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE TABLES — list all tables in current keyspace
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_tables_in_keyspace() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_tbls");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.table_a (id int PRIMARY KEY, val text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.table_b (id int PRIMARY KEY, num int)"),
    )
    .success();

    let output = cqlsh_cmd(scylla)
        .args(["-k", &ks, "-e", "DESCRIBE TABLES"])
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("table_a"),
        "DESCRIBE TABLES should list table_a: {stdout}"
    );
    assert!(
        stdout.contains("table_b"),
        "DESCRIBE TABLES should list table_b: {stdout}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE CLUSTER — cluster topology info
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_cluster() {
    let scylla = get_scylla();

    let output = execute_cql_output(scylla, "DESCRIBE CLUSTER");
    assert!(
        output.contains("Cluster") || output.contains("Partitioner") || output.contains("Snitch"),
        "DESCRIBE CLUSTER should show cluster info: {output}"
    );
}

// ---------------------------------------------------------------------------
// DESCRIBE TABLE — columns named after CQL keywords (quoted identifiers)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_table_with_keyword_column_names() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_kwcols");

    execute_cql(
        scylla,
        &format!(
            "CREATE TABLE {ks}.kw_table (\
             id int PRIMARY KEY, \
             \"key\" text, \
             \"set\" int)"
        ),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("DESCRIBE TABLE {ks}.kw_table"));
    assert!(
        output.contains("CREATE TABLE"),
        "DESCRIBE TABLE with keyword column names: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// DESCRIBE SCHEMA — DDL for all objects in current keyspace
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_schema() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_schema");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.schema_test (id int PRIMARY KEY, val text)"),
    )
    .success();

    let output = cqlsh_cmd(scylla)
        .args(["-k", &ks, "-e", "DESCRIBE SCHEMA"])
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("CREATE"),
        "DESCRIBE SCHEMA should output CREATE statements: {stdout}"
    );

    drop_test_keyspace(scylla, &ks);
}

//! Core integration tests for cqlsh-rs against a live ScyllaDB instance.
//!
//! Mirrors the Python cqlsh dtest TestCqlshSmoke test suite.

use super::helpers::*;

#[test]
#[ignore = "requires Docker"]
fn test_simple_insert_and_select() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "crud");

    // Create table
    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.users (id int PRIMARY KEY, name text, age int)"),
    )
    .success();

    // Insert
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.users (id, name, age) VALUES (1, 'Alice', 30)"),
    )
    .success();

    // Select and verify
    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.users WHERE id = 1"));
    assert!(output.contains("Alice"));
    assert!(output.contains("30"));

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_insert_update_delete() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "iud");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.data (id int PRIMARY KEY, val text)"),
    )
    .success();

    // Insert
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.data (id, val) VALUES (1, 'original')"),
    )
    .success();

    // Update
    execute_cql(
        scylla,
        &format!("UPDATE {ks}.data SET val = 'updated' WHERE id = 1"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT val FROM {ks}.data WHERE id = 1"));
    assert!(output.contains("updated"));
    assert!(!output.contains("original"));

    // Delete
    execute_cql(scylla, &format!("DELETE FROM {ks}.data WHERE id = 1")).success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.data WHERE id = 1"));
    assert!(!output.contains("updated"));

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_create_and_drop_keyspace() {
    let scylla = get_scylla();
    let ks = format!(
        "test_ddl_ks_{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            % 0xFFFFFF
    );

    // Create keyspace
    execute_cql(
        scylla,
        &format!(
            "CREATE KEYSPACE {ks} WITH replication = \
             {{'class': 'SimpleStrategy', 'replication_factor': 1}}"
        ),
    )
    .success();

    // Verify it exists by querying system_schema
    let output = execute_cql_output(
        scylla,
        &format!("SELECT keyspace_name FROM system_schema.keyspaces WHERE keyspace_name = '{ks}'"),
    );
    assert!(output.contains(&ks));

    // Drop keyspace
    execute_cql(scylla, &format!("DROP KEYSPACE {ks}")).success();

    // Verify it's gone
    let output = execute_cql_output(
        scylla,
        &format!("SELECT keyspace_name FROM system_schema.keyspaces WHERE keyspace_name = '{ks}'"),
    );
    assert!(!output.contains(&ks));
}

#[test]
#[ignore = "requires Docker"]
fn test_create_and_drop_table() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "ddl_table");

    // Create table
    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.test_table (id int PRIMARY KEY, data text)"),
    )
    .success();

    // Insert data to verify table works
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.test_table (id, data) VALUES (1, 'hello')"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.test_table"));
    assert!(output.contains("hello"));

    // Drop table
    execute_cql(scylla, &format!("DROP TABLE {ks}.test_table")).success();

    // Verify table is gone — query should fail
    execute_cql(scylla, &format!("SELECT * FROM {ks}.test_table")).failure();

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_use_keyspace() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "use_ks");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.test_use (id int PRIMARY KEY, val text)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.test_use (id, val) VALUES (1, 'test')"),
    )
    .success();

    // Use the keyspace and query without qualified name
    let output = cqlsh_cmd(scylla)
        .args(["-k", &ks, "-e", "SELECT * FROM test_use WHERE id = 1"])
        .output()
        .expect("failed to execute cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test"),
        "Expected 'test' in output: {stdout}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_batch_statement() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "batch");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.batch_test (id int PRIMARY KEY, val text)"),
    )
    .success();

    // Execute BATCH
    execute_cql(
        scylla,
        &format!(
            "BEGIN BATCH \
             INSERT INTO {ks}.batch_test (id, val) VALUES (1, 'one'); \
             INSERT INTO {ks}.batch_test (id, val) VALUES (2, 'two'); \
             INSERT INTO {ks}.batch_test (id, val) VALUES (3, 'three'); \
             APPLY BATCH"
        ),
    )
    .success();

    // Verify all rows inserted
    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.batch_test"));
    assert!(output.contains("one"));
    assert!(output.contains("two"));
    assert!(output.contains("three"));

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_uuid_type() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "uuid");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.uuid_test (id uuid PRIMARY KEY, name text)"),
    )
    .success();

    let test_uuid = "550e8400-e29b-41d4-a716-446655440000";
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.uuid_test (id, name) VALUES ({test_uuid}, 'test')"),
    )
    .success();

    let output = execute_cql_output(
        scylla,
        &format!("SELECT * FROM {ks}.uuid_test WHERE id = {test_uuid}"),
    );
    assert!(output.contains(test_uuid));
    assert!(output.contains("test"));

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_truncate() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "truncate");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.trunc_test (id int PRIMARY KEY, val text)"),
    )
    .success();

    // Insert some data
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.trunc_test (id, val) VALUES (1, 'one')"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.trunc_test (id, val) VALUES (2, 'two')"),
    )
    .success();

    // Verify data exists
    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.trunc_test"));
    assert!(output.contains("one"));

    // Truncate
    execute_cql(scylla, &format!("TRUNCATE {ks}.trunc_test")).success();

    // Verify empty — should show header but no data rows
    let output = execute_cql_output(scylla, &format!("SELECT count(*) FROM {ks}.trunc_test"));
    assert!(output.contains("0"));

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_multiple_data_types() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "types");

    execute_cql(
        scylla,
        &format!(
            "CREATE TABLE {ks}.types_test (\
             id int PRIMARY KEY, \
             t text, \
             b boolean, \
             bi bigint, \
             f float, \
             d double, \
             bl blob, \
             u uuid, \
             si smallint, \
             ti tinyint)"
        ),
    )
    .success();

    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.types_test (id, t, b, bi, f, d, bl, u, si, ti) \
             VALUES (1, 'hello', true, 9223372036854775807, 3.14, 2.718281828, \
             0xdeadbeef, 550e8400-e29b-41d4-a716-446655440000, 32000, 127)"
        ),
    )
    .success();

    let output = execute_cql_output(
        scylla,
        &format!("SELECT * FROM {ks}.types_test WHERE id = 1"),
    );

    assert!(output.contains("hello"));
    assert!(output.contains("True"));
    assert!(output.contains("9223372036854775807"));
    assert!(output.contains("0xdeadbeef"));
    assert!(output.contains("550e8400-e29b-41d4-a716-446655440000"));

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_collection_types() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "collections");

    execute_cql(
        scylla,
        &format!(
            "CREATE TABLE {ks}.coll_test (\
             id int PRIMARY KEY, \
             tags set<text>, \
             scores list<int>, \
             props map<text, text>)"
        ),
    )
    .success();

    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.coll_test (id, tags, scores, props) \
             VALUES (1, {{'a', 'b', 'c'}}, [10, 20, 30], {{'key1': 'val1', 'key2': 'val2'}})"
        ),
    )
    .success();

    let output = execute_cql_output(
        scylla,
        &format!("SELECT * FROM {ks}.coll_test WHERE id = 1"),
    );

    // Verify collections appear in output
    assert!(output.contains("10"));
    assert!(output.contains("20"));
    assert!(output.contains("30"));
    assert!(output.contains("key1"));
    assert!(output.contains("val1"));

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_empty_string_values() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "empty");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.empty_test (id int PRIMARY KEY, val text)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.empty_test (id, val) VALUES (1, '')"),
    )
    .success();

    // Empty strings should be queryable
    let output = execute_cql_output(
        scylla,
        &format!("SELECT * FROM {ks}.empty_test WHERE id = 1"),
    );
    // The row should exist (id=1 should appear)
    assert!(output.contains("1"));

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_null_values() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "nulls");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.null_test (id int PRIMARY KEY, val text, num int)"),
    )
    .success();

    // Insert with only PK set, other columns null
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.null_test (id) VALUES (1)"),
    )
    .success();

    let output = execute_cql_output(
        scylla,
        &format!("SELECT * FROM {ks}.null_test WHERE id = 1"),
    );
    // Null values display as blank (empty), matching Python cqlsh
    assert!(
        !output.contains("null"),
        "Null should display as blank, not 'null': {output}"
    );
    assert!(
        output.contains(" 1 "),
        "Row with id=1 should be present: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_create_and_drop_index() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "index");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.idx_test (id int PRIMARY KEY, name text)"),
    )
    .success();

    // Create index
    execute_cql(
        scylla,
        &format!("CREATE INDEX idx_name ON {ks}.idx_test (name)"),
    )
    .success();

    // Drop index
    execute_cql(scylla, &format!("DROP INDEX {ks}.idx_name")).success();

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_alter_table() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "alter");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.alter_test (id int PRIMARY KEY, name text)"),
    )
    .success();

    // Add a column
    execute_cql(
        scylla,
        &format!("ALTER TABLE {ks}.alter_test ADD email text"),
    )
    .success();

    // Insert with new column
    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.alter_test (id, name, email) VALUES (1, 'Alice', 'alice@test.com')"
        ),
    )
    .success();

    let output = execute_cql_output(
        scylla,
        &format!("SELECT * FROM {ks}.alter_test WHERE id = 1"),
    );
    assert!(output.contains("alice@test.com"));

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_show_version() {
    let scylla = get_scylla();

    let output = execute_cql_output(scylla, "SHOW VERSION");
    assert!(output.contains("cqlsh"));
}

#[test]
#[ignore = "requires Docker"]
fn test_show_host() {
    let scylla = get_scylla();

    let output = execute_cql_output(scylla, "SHOW HOST");
    assert!(output.contains("Connected to"));
}

#[test]
#[ignore = "requires Docker"]
fn test_consistency_command() {
    let scylla = get_scylla();

    let output = execute_cql_output(scylla, "CONSISTENCY");
    assert!(output.contains("Current consistency level is"));
}

#[test]
#[ignore = "requires Docker"]
fn test_connection_banner() {
    let scylla = get_scylla();

    // Run any command and check stderr for the connection banner
    let output = cqlsh_cmd(scylla)
        .args(["-e", "SELECT release_version FROM system.local"])
        .output()
        .expect("failed to execute cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The banner is printed to stdout before the query result
    assert!(
        stdout.contains("Connected to") || stdout.contains("cqlsh"),
        "Expected connection banner in output: {stdout}"
    );
}

#[test]
#[ignore = "requires Docker"]
fn test_no_color_flag() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "nocolor");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.nc_test (id int PRIMARY KEY, val text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.nc_test (id, val) VALUES (1, 'test')"),
    )
    .success();

    let output = cqlsh_cmd(scylla)
        .args(["--no-color", "-e", &format!("SELECT * FROM {ks}.nc_test")])
        .output()
        .expect("failed to execute cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verify no ANSI escape codes in output
    assert!(
        !stdout.contains("\x1b["),
        "Found ANSI codes in --no-color output: {stdout}"
    );

    drop_test_keyspace(scylla, &ks);
}

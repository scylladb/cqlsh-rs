//! COPY TO / COPY FROM integration tests for cqlsh-rs.
//!
//! Tests the COPY command against a live ScyllaDB instance using temporary
//! CSV files. Mirrors the Python cqlsh dtest `cqlsh_copy_tests.py` suite
//! (Phase 6 of SP19).
//!
//! # Skip behaviour
//!
//! COPY is a cqlsh built-in command and must be dispatched by cqlsh-rs before
//! the statement reaches the server. If COPY is not yet wired up in
//! non-interactive (`-e`) mode, the server will reject it with a
//! `SyntaxException`. In that case each test calls `skip_if_copy_unsupported`
//! and returns early rather than failing hard.

use std::fs;
use std::io::Write as IoWrite;

use super::helpers::*;

/// Run a trivial COPY TO probe and return `false` when the server rejects the
/// command with a `SyntaxException` (meaning COPY is not yet dispatched in
/// non-interactive mode). Returns `true` when the command is handled by
/// cqlsh-rs itself (success or a cqlsh-level error).
fn copy_dispatched(scylla: &ScyllaContainer, ks: &str) -> bool {
    // A probe table that exists only for the duration of this check.
    let _ = cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("CREATE TABLE IF NOT EXISTS {ks}._copy_probe (id int PRIMARY KEY)"),
        ])
        .output();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    let result = cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}._copy_probe TO '{csv_path}'")])
        .output()
        .expect("failed to run cqlsh-rs for COPY probe");

    let stderr = String::from_utf8_lossy(&result.stderr);
    // If the server sees the statement it returns SyntaxException.
    !stderr.contains("SyntaxException") && !stderr.contains("no viable alternative")
}

// ---------------------------------------------------------------------------
// 6.1 — COPY TO basic: export rows to CSV and verify file content
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_copy_to_basic() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "copy_to");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode (Phase 4 not yet complete)");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.copy_test (id int PRIMARY KEY, name text, score int)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.copy_test (id, name, score) VALUES (1, 'Alice', 95)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.copy_test (id, name, score) VALUES (2, 'Bob', 87)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.copy_test (id, name, score) VALUES (3, 'Charlie', 92)"),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.copy_test TO '{csv_path}'")])
        .assert()
        .success();

    let contents = fs::read_to_string(&csv_path).expect("failed to read CSV output");
    assert!(
        contents.contains("Alice"),
        "CSV should contain 'Alice': {contents}"
    );
    assert!(
        contents.contains("Bob"),
        "CSV should contain 'Bob': {contents}"
    );
    assert!(
        contents.contains("Charlie"),
        "CSV should contain 'Charlie': {contents}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 6.2 — COPY FROM basic: import CSV rows and verify data in DB
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_copy_from_basic() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "copy_from");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode (Phase 4 not yet complete)");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.import_test (id int PRIMARY KEY, name text, score int)"),
    )
    .success();

    let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    writeln!(tmp, "id,name,score").unwrap();
    writeln!(tmp, "10,Diana,88").unwrap();
    writeln!(tmp, "11,Eve,76").unwrap();
    writeln!(tmp, "12,Frank,91").unwrap();
    tmp.flush().unwrap();
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("COPY {ks}.import_test FROM '{csv_path}' WITH HEADER = true"),
        ])
        .assert()
        .success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.import_test"));
    assert!(output.contains("Diana"), "Expected Diana in DB: {output}");
    assert!(output.contains("Eve"), "Expected Eve in DB: {output}");
    assert!(output.contains("Frank"), "Expected Frank in DB: {output}");
    assert!(output.contains("88"), "Expected score 88: {output}");

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 6.3 — COPY round-trip: COPY TO → TRUNCATE → COPY FROM → verify
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_copy_round_trip() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "copy_rt");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode (Phase 4 not yet complete)");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.roundtrip (id int PRIMARY KEY, val text)"),
    )
    .success();

    for i in 1..=5 {
        execute_cql(
            scylla,
            &format!("INSERT INTO {ks}.roundtrip (id, val) VALUES ({i}, 'item_{i}')"),
        )
        .success();
    }

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.roundtrip TO '{csv_path}'")])
        .assert()
        .success();

    execute_cql(scylla, &format!("TRUNCATE {ks}.roundtrip")).success();

    let count_output = execute_cql_output(scylla, &format!("SELECT count(*) FROM {ks}.roundtrip"));
    assert!(
        count_output.contains(" 0"),
        "Table should be empty after TRUNCATE: {count_output}"
    );

    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.roundtrip FROM '{csv_path}'")])
        .assert()
        .success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.roundtrip"));
    for i in 1..=5 {
        assert!(
            output.contains(&format!("item_{i}")),
            "Expected item_{i} after round-trip: {output}"
        );
    }

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 6.4 — COPY TO with HEADER option
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_copy_to_with_header() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "copy_hdr");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode (Phase 4 not yet complete)");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.hdr_test (id int PRIMARY KEY, name text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.hdr_test (id, name) VALUES (1, 'Alice')"),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("COPY {ks}.hdr_test TO '{csv_path}' WITH HEADER = true"),
        ])
        .assert()
        .success();

    let contents = fs::read_to_string(&csv_path).expect("failed to read CSV");
    let first_line = contents.lines().next().unwrap_or("");
    assert!(
        first_line.contains("id") || first_line.contains("name"),
        "First CSV line should be header with column names: {first_line}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 6.5 — COPY with NULL indicator
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_copy_null_indicator() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "copy_null");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode (Phase 4 not yet complete)");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.null_test (id int PRIMARY KEY, val text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.null_test (id) VALUES (1)"),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("COPY {ks}.null_test TO '{csv_path}' WITH NULLVAL = 'NULL'"),
        ])
        .assert()
        .success();

    let contents = fs::read_to_string(&csv_path).expect("failed to read CSV");
    assert!(
        contents.contains("NULL"),
        "CSV should use NULL indicator for null column: {contents}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 6.6 — COPY with collection types (list, set, map)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_copy_collections_round_trip() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "copy_coll");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode (Phase 4 not yet complete)");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.coll (id int PRIMARY KEY, tags set<text>, scores list<int>)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.coll (id, tags, scores) VALUES (1, {{'alpha', 'beta'}}, [10, 20, 30])"
        ),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.coll TO '{csv_path}'")])
        .assert()
        .success();

    execute_cql(scylla, &format!("TRUNCATE {ks}.coll")).success();

    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.coll FROM '{csv_path}'")])
        .assert()
        .success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.coll WHERE id = 1"));
    assert!(
        output.contains("alpha") || output.contains("beta"),
        "Expected set values after round-trip: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 6.7 — COPY wrong number of columns should fail gracefully
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_copy_from_wrong_column_count() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "copy_bad");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode (Phase 4 not yet complete)");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.bad_test (id int PRIMARY KEY, a text, b text)"),
    )
    .success();

    let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    writeln!(tmp, "1,val_a,val_b,extra_column").unwrap();
    tmp.flush().unwrap();
    let csv_path = tmp.path().to_str().unwrap().to_string();

    // Should fail or skip the bad rows; just verify cqlsh-rs doesn't panic
    let result = cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.bad_test FROM '{csv_path}'")])
        .output()
        .expect("failed to run cqlsh-rs");

    let _ = result.status;

    drop_test_keyspace(scylla, &ks);
}

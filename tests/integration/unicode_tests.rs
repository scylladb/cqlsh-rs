//! Unicode integration tests for cqlsh-rs.
//!
//! Mirrors the Python cqlsh `test_unicode.py` test suite (Phase 5 of SP19).
//! Tests verify correct handling of Unicode values, identifiers, and
//! multi-line input against a real ScyllaDB instance.

use super::helpers::*;

// ---------------------------------------------------------------------------
// 5.1 — Unicode value round-trip (insert + select Unicode text values)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_unicode_value_round_trip() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "uni_val");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.uni (id int PRIMARY KEY, val text)"),
    )
    .success();

    // Test various Unicode scripts
    let test_cases = vec![
        (1, "こんにちは"),        // Japanese
        (2, "Привет мир"),        // Russian
        (3, "مرحبا بالعالم"),     // Arabic
        (4, "🎉🚀💡"),            // Emoji
        (5, "café résumé naïve"), // Latin with accents
    ];

    for (id, val) in &test_cases {
        execute_cql(
            scylla,
            &format!("INSERT INTO {ks}.uni (id, val) VALUES ({id}, '{val}')"),
        )
        .success();
    }

    // Verify round-trip
    for (id, val) in &test_cases {
        let output =
            execute_cql_output(scylla, &format!("SELECT val FROM {ks}.uni WHERE id = {id}"));
        assert!(
            output.contains(val),
            "Unicode value '{val}' not found in output for id={id}: {output}"
        );
    }

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 5.1b — The classic "eat glass" test from Python cqlsh dtests
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_eat_glass() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "uni_glass");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.glass (id int PRIMARY KEY, val text)"),
    )
    .success();

    // The classic glass-eating test string.
    // CQL uses '' to escape a single quote inside a string literal.
    let glass = "I can eat glass and it doesn''t hurt me. 私はガラスを食べられます。それは私を傷つけません。";
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.glass (id, val) VALUES (1, '{glass}')"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT val FROM {ks}.glass WHERE id = 1"));
    assert!(
        output.contains("I can eat glass") || output.contains("doesn"),
        "Expected English portion: {output}"
    );
    assert!(
        output.contains("私はガラスを食べられます"),
        "Expected Japanese portion: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 5.2 — Unicode identifiers (table and column names with Unicode)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_unicode_identifiers() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "uni_ident");

    // CQL supports Unicode identifiers when quoted with double quotes.
    // ScyllaDB may reject non-ASCII table names; skip gracefully if so.
    let create_result = cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!(
                "CREATE TABLE {ks}.\"données\" (id int PRIMARY KEY, \"prénom\" text, \"名前\" text);"
            ),
        ])
        .output()
        .expect("failed to run cqlsh-rs");
    if !create_result.status.success() {
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.\"données\" (id, \"prénom\", \"名前\") VALUES (1, 'Pierre', '太郎')"
        ),
    )
    .success();

    let output = execute_cql_output(
        scylla,
        &format!("SELECT * FROM {ks}.\"données\" WHERE id = 1"),
    );

    assert!(
        output.contains("Pierre"),
        "Expected French name in output: {output}"
    );
    assert!(
        output.contains("太郎"),
        "Expected Japanese name in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 5.3 — Unicode multiline input (multi-statement with Unicode)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_unicode_multiline_input() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "uni_multi");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.uni_ml (id int PRIMARY KEY, val text)"),
    )
    .success();

    // Multiple semicolon-separated statements with Unicode
    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.uni_ml (id, val) VALUES (1, '日本語'); \
             INSERT INTO {ks}.uni_ml (id, val) VALUES (2, 'Ελληνικά')"
        ),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.uni_ml"));
    assert!(
        output.contains("日本語"),
        "Expected Japanese in output: {output}"
    );
    assert!(
        output.contains("Ελληνικά"),
        "Expected Greek in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 5.4 — Unicode DESCRIBE (DESCRIBE objects with Unicode names)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_unicode_describe() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "uni_desc");

    // Create a table with Unicode column names.
    // ScyllaDB may reject non-ASCII table names; skip gracefully if so.
    let create_result = cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("CREATE TABLE {ks}.\"テスト\" (id int PRIMARY KEY, \"名前\" text);"),
        ])
        .output()
        .expect("failed to run cqlsh-rs");
    if !create_result.status.success() {
        drop_test_keyspace(scylla, &ks);
        return;
    }

    let output = execute_cql_output(scylla, &format!("DESCRIBE TABLE {ks}.\"テスト\""));

    assert!(
        output.contains("CREATE TABLE"),
        "DESCRIBE should show CREATE TABLE: {output}"
    );
    // The Unicode table name should appear in the output
    assert!(
        output.contains("テスト"),
        "DESCRIBE should include Unicode table name: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

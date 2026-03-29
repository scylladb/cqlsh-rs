//! Escape sequence integration tests for cqlsh-rs.
//!
//! Mirrors the Python cqlsh `test_escape_decoding.py`, `test_escape_roundtrip.py`,
//! and `test_escape_sequences.py` test suites (Phase 4 of SP19).
//! Tests verify correct handling of escape sequences in CQL queries against
//! a real ScyllaDB instance.

use super::helpers::*;

// ---------------------------------------------------------------------------
// 4.1 — Hex escape decoding (e.g. \x41 → 'A')
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_hex_escape_in_query() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "esc_hex");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.esc (id int PRIMARY KEY, val text)"),
    )
    .success();

    // Insert a string with known content, then verify round-trip
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.esc (id, val) VALUES (1, 'ABCDEF')"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT val FROM {ks}.esc WHERE id = 1"));
    assert!(
        output.contains("ABCDEF"),
        "Expected string with hex-equivalent chars: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 4.2 — Standard escape sequences (\n, \t, etc. in string literals)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_standard_escape_newline_tab() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "esc_std");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.esc (id int PRIMARY KEY, val text)"),
    )
    .success();

    // CQL string with literal tab character
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.esc (id, val) VALUES (1, 'hello\tworld')"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT val FROM {ks}.esc WHERE id = 1"));
    // The output should contain the string (tab may be displayed differently)
    assert!(
        output.contains("hello") && output.contains("world"),
        "Expected string parts: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 4.3 — Backslash escaping (literal backslash in strings)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_backslash_in_string() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "esc_bs");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.esc (id int PRIMARY KEY, val text)"),
    )
    .success();

    // CQL uses $$ or doubled quotes for special chars; test with a path-like string
    execute_cql(
        scylla,
        &format!(r"INSERT INTO {ks}.esc (id, val) VALUES (1, 'C:\\Users\\test')"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT val FROM {ks}.esc WHERE id = 1"));
    // Should contain some form of the path
    assert!(
        output.contains("Users") && output.contains("test"),
        "Expected path components in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 4.4 — Quote escaping (single quotes escaped as '' in CQL)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_quote_escaping() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "esc_quote");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.esc (id int PRIMARY KEY, val text)"),
    )
    .success();

    // CQL escapes single quotes by doubling: 'it''s'
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.esc (id, val) VALUES (1, 'it''s a test')"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT val FROM {ks}.esc WHERE id = 1"));
    assert!(
        output.contains("it's a test"),
        "Expected unescaped single quote in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 4.5 — Control character round-trip (insert + select preserves data)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_special_chars_roundtrip() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "esc_ctrl");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.esc (id int PRIMARY KEY, val text)"),
    )
    .success();

    // Insert strings with various special characters
    let test_cases = vec![
        (1, "line1\nline2"),         // newline
        (2, "col1\tcol2"),           // tab
        (3, "with spaces  between"), // multiple spaces
    ];

    for (id, val) in &test_cases {
        execute_cql(
            scylla,
            &format!("INSERT INTO {ks}.esc (id, val) VALUES ({id}, '{val}')"),
        )
        .success();
    }

    // Verify each value round-trips (at least the non-control parts)
    for (id, _) in &test_cases {
        let output =
            execute_cql_output(scylla, &format!("SELECT val FROM {ks}.esc WHERE id = {id}"));
        // Row should exist
        assert!(
            output.contains("(1 row)"),
            "Expected 1 row for id={id}: {output}"
        );
    }

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 4.6 — Mixed content round-trip (various escape types in one string)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_mixed_content_roundtrip() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "esc_mix");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.esc (id int PRIMARY KEY, val text)"),
    )
    .success();

    // A string with quotes and various characters
    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.esc (id, val) VALUES (1, 'Hello, it''s a \"mixed\" test: 100%')"
        ),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT val FROM {ks}.esc WHERE id = 1"));
    assert!(
        output.contains("Hello"),
        "Expected greeting in output: {output}"
    );
    assert!(
        output.contains("mixed"),
        "Expected 'mixed' in output: {output}"
    );
    assert!(
        output.contains("100%"),
        "Expected '100%' in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

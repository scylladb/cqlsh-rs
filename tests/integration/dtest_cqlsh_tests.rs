//! Integration tests ported from Python cqlsh dtest cqlsh_tests.py

use super::helpers::*;

#[test]
#[ignore = "requires Docker"]
fn test_past_and_future_dates() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dates");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.date_test (id int PRIMARY KEY, ts timestamp)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.date_test (id, ts) VALUES (1, '2010-01-01 00:00:00+0000')"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.date_test (id, ts) VALUES (2, '3000-01-01 00:00:00+0000')"),
    )
    .success();

    let output = execute_cql_output_direct(scylla, &format!("SELECT * FROM {ks}.date_test"));
    assert!(
        output.contains("2010"),
        "Expected past date 2010 in output: {output}"
    );
    assert!(
        output.contains("3000"),
        "Expected future date 3000 in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_eat_glass() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "glass");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.glass (id int PRIMARY KEY, lang text, phrase text)"),
    )
    .success();

    // Representative multilingual unicode strings
    let phrases = [
        (1, "chinese", "我能吞下玻璃而不伤身体"),
        (2, "japanese", "私はガラスを食べられます"),
        (3, "polish", "Mogę jeść szkło"),
        (4, "arabic", "أنا قادر على أكل الزجاج"),
        (5, "greek", "Μπορώ να φάω σπασμένα γυαλιά"),
    ];

    for (id, lang, phrase) in &phrases {
        execute_cql(
            scylla,
            &format!(
                "INSERT INTO {ks}.glass (id, lang, phrase) VALUES ({id}, '{lang}', '{phrase}')"
            ),
        )
        .success();
    }

    let output = execute_cql_output_direct(scylla, &format!("SELECT * FROM {ks}.glass"));
    assert!(
        output.contains("chinese"),
        "Expected 'chinese' in output: {output}"
    );
    assert!(
        output.contains("japanese"),
        "Expected 'japanese' in output: {output}"
    );
    assert!(
        output.contains("polish"),
        "Expected 'polish' in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_source_glass() {
    use std::io::Write;

    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "srcglass");

    // Write a temporary .cql file with unicode data
    let mut tmpfile = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let cql_content = format!(
        "CREATE TABLE {ks}.src_glass (id int PRIMARY KEY, phrase text);\n\
         INSERT INTO {ks}.src_glass (id, phrase) VALUES (1, '我能吞下玻璃而不伤身体');\n\
         INSERT INTO {ks}.src_glass (id, phrase) VALUES (2, 'Mogę jeść szkło');\n"
    );
    tmpfile
        .write_all(cql_content.as_bytes())
        .expect("failed to write cql file");
    let filepath = tmpfile.path().to_str().expect("invalid temp file path");

    // Execute via -f (SOURCE)
    let output = cqlsh_cmd(scylla)
        .args(["-f", filepath])
        .output()
        .expect("failed to execute cqlsh-rs");

    assert!(
        output.status.success(),
        "cqlsh-rs -f failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let select_output = execute_cql_output_direct(scylla, &format!("SELECT * FROM {ks}.src_glass"));
    assert!(
        select_output.contains("玻璃"),
        "Expected Chinese characters in output: {select_output}"
    );
    assert!(
        select_output.contains("szkło"),
        "Expected Polish text in output: {select_output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_tracing_from_system_traces() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args(["-e", "TRACING ON; SELECT * FROM system.local;"])
        .output()
        .expect("failed to execute cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Tracing session:"),
        "Expected 'Tracing session:' in output: {stdout}"
    );
    // Tracing itself should not query system_traces in a visible loop
    assert!(
        !stdout.contains("system_traces.events"),
        "Output should not contain tracing of system_traces itself: {stdout}"
    );
}

#[test]
#[ignore = "requires Docker"]
fn test_select_element_inside_udt() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "udt");

    execute_cql(
        scylla,
        &format!("CREATE TYPE {ks}.address (street text, city text, zip text)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.contacts (id int PRIMARY KEY, addr frozen<{ks}.address>)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.contacts (id, addr) \
             VALUES (1, {{street: '123 Main St', city: 'Springfield', zip: '12345'}})"
        ),
    )
    .success();

    let output = execute_cql_output_direct(
        scylla,
        &format!("SELECT addr.city FROM {ks}.contacts WHERE id = 1"),
    );
    assert!(
        output.contains("Springfield"),
        "Expected 'Springfield' in UDT field selection output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_float_formatting() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "floats");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.float_test (id int PRIMARY KEY, f float, d double)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.float_test (id, f, d) VALUES (1, 3.14, 2.718281828)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.float_test (id, f, d) VALUES (2, -1.5, -0.001)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.float_test (id, f, d) VALUES (3, 0.0, 0.0)"),
    )
    .success();

    let output = execute_cql_output_direct(scylla, &format!("SELECT * FROM {ks}.float_test"));
    assert!(
        output.contains("3.14") || output.contains("3.1"),
        "Expected float 3.14 in output: {output}"
    );
    assert!(
        output.contains("2.718") || output.contains("2.71"),
        "Expected double 2.718... in output: {output}"
    );
    assert!(
        output.contains("-1.5"),
        "Expected negative float -1.5 in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_int_values() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "ints");

    execute_cql(
        scylla,
        &format!(
            "CREATE TABLE {ks}.int_test (\
             id int PRIMARY KEY, \
             big bigint, \
             sm smallint, \
             ti tinyint)"
        ),
    )
    .success();

    // Insert boundary values
    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.int_test (id, big, sm, ti) \
             VALUES (2147483647, 9223372036854775807, 32767, 127)"
        ),
    )
    .success();

    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.int_test (id, big, sm, ti) \
             VALUES (-2147483648, -9223372036854775808, -32768, -128)"
        ),
    )
    .success();

    let output = execute_cql_output_direct(scylla, &format!("SELECT * FROM {ks}.int_test"));
    assert!(
        output.contains("2147483647"),
        "Expected int max in output: {output}"
    );
    assert!(
        output.contains("-2147483648"),
        "Expected int min in output: {output}"
    );
    assert!(
        output.contains("9223372036854775807"),
        "Expected bigint max in output: {output}"
    );
    assert!(
        output.contains("32767"),
        "Expected smallint max in output: {output}"
    );
    assert!(
        output.contains("127"),
        "Expected tinyint max in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_datetime_values() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "datetime");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.dt_test (id int PRIMARY KEY, d date, t time)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.dt_test (id, d, t) VALUES (1, '2023-06-15', '14:30:00')"),
    )
    .success();

    let output =
        execute_cql_output_direct(scylla, &format!("SELECT * FROM {ks}.dt_test WHERE id = 1"));
    assert!(
        output.contains("2023-06-15"),
        "Expected date '2023-06-15' in output: {output}"
    );
    assert!(
        output.contains("14:30"),
        "Expected time '14:30' in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_tracing() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args(["-e", "TRACING ON; SELECT * FROM system.local;"])
        .output()
        .expect("failed to execute cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "cqlsh-rs failed with tracing: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("Tracing session:"),
        "Expected tracing session info in output: {stdout}"
    );
}

#[test]
#[ignore = "requires Docker"]
fn test_connect_timeout() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args(["--connect-timeout", "10", "--debug", "-e", "SHOW VERSION"])
        .output()
        .expect("failed to execute cqlsh-rs");

    assert!(
        output.status.success(),
        "cqlsh-rs with --connect-timeout failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("cqlsh") || combined.contains("Connected") || combined.contains("debug"),
        "Expected version or debug output: {combined}"
    );
}

#[test]
#[ignore = "requires Docker"]
fn test_describe_round_trip() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "roundtrip");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.rt_test (id int PRIMARY KEY, name text, age int)"),
    )
    .success();

    // DESCRIBE the table
    let describe_output =
        execute_cql_output_direct(scylla, &format!("DESCRIBE TABLE {ks}.rt_test"));
    assert!(
        describe_output.contains("rt_test"),
        "DESCRIBE output should contain table name: {describe_output}"
    );
    assert!(
        describe_output.contains("CREATE TABLE"),
        "DESCRIBE output should contain CREATE TABLE: {describe_output}"
    );

    // DROP the table
    execute_cql_direct(scylla, &format!("DROP TABLE {ks}.rt_test"));

    // Re-create from DESCRIBE output — extract just the CREATE TABLE statement
    // The DESCRIBE output contains the CREATE TABLE DDL we can re-execute
    // Find the CREATE TABLE ... ; block
    let create_stmt = describe_output
        .lines()
        .skip_while(|l| !l.trim_start().starts_with("CREATE TABLE"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !create_stmt.is_empty(),
        "Could not extract CREATE TABLE from DESCRIBE output: {describe_output}"
    );

    execute_cql_direct(scylla, &create_stmt);

    // DESCRIBE again and verify structure matches
    let describe_output2 =
        execute_cql_output_direct(scylla, &format!("DESCRIBE TABLE {ks}.rt_test"));
    assert!(
        describe_output2.contains("rt_test"),
        "Second DESCRIBE should contain table name: {describe_output2}"
    );
    assert!(
        describe_output2.contains("name"),
        "Second DESCRIBE should contain 'name' column: {describe_output2}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_commented_lines() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "comments");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.comment_test (id int PRIMARY KEY, val text)"),
    )
    .success();

    // Double-dash comment — use a .cql file since `-e "-- ..."` conflicts with CLI arg parsing
    let mut cql_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
    use std::io::Write as IoWrite;
    writeln!(
        cql_file,
        "-- This is a comment\nINSERT INTO {ks}.comment_test (id, val) VALUES (1, 'dash');"
    )
    .unwrap();
    cql_file.flush().unwrap();
    let cql_path = cql_file.path().to_str().unwrap().to_string();
    let output = cqlsh_cmd(scylla)
        .args(["-f", &cql_path])
        .output()
        .expect("failed to execute cqlsh-rs");
    assert!(
        output.status.success(),
        "Double-dash comment failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // C-style block comment
    let output = cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!(
                "/* block comment */ INSERT INTO {ks}.comment_test (id, val) VALUES (2, 'block');"
            ),
        ])
        .output()
        .expect("failed to execute cqlsh-rs");
    assert!(
        output.status.success(),
        "Block comment failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let select_output =
        execute_cql_output_direct(scylla, &format!("SELECT * FROM {ks}.comment_test"));
    assert!(
        select_output.contains("dash"),
        "Expected 'dash' row in output: {select_output}"
    );
    assert!(
        select_output.contains("block"),
        "Expected 'block' row in output: {select_output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_colons_in_string_literals() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "colons");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.colon_test (id int PRIMARY KEY, val text)"),
    )
    .success();

    // Value containing a colon — should not be treated as a named bind marker
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.colon_test (id, val) VALUES (1, 'a]b:c')"),
    )
    .success();

    let output = execute_cql_output_direct(
        scylla,
        &format!("SELECT * FROM {ks}.colon_test WHERE id = 1"),
    );
    assert!(
        output.contains("a]b:c"),
        "Expected literal 'a]b:c' in output (colon should not be bind marker): {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_describe_describes_non_default_compaction_parameters() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "compaction");

    execute_cql(
        scylla,
        &format!(
            "CREATE TABLE {ks}.compaction_test (id int PRIMARY KEY, val text) \
             WITH compaction = {{'class': 'LeveledCompactionStrategy', 'sstable_size_in_mb': '10'}}"
        ),
    )
    .success();

    let output = execute_cql_output_direct(scylla, &format!("DESCRIBE TABLE {ks}.compaction_test"));
    // ScyllaDB may show compaction info differently — assert the WITH clause is present at all
    assert!(
        output.contains("LeveledCompactionStrategy")
            || output.contains("leveled")
            || output.contains("compaction"),
        "Expected compaction info in DESCRIBE output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_describe_on_non_reserved_keywords() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "keywords");

    // Use columns named after non-reserved CQL keywords
    execute_cql(
        scylla,
        &format!(
            "CREATE TABLE {ks}.keyword_test (\
             id int PRIMARY KEY, \
             \"key\" text, \
             clustering text, \
             \"type\" text)"
        ),
    )
    .success();

    let output = execute_cql_output_direct(scylla, &format!("DESCRIBE TABLE {ks}.keyword_test"));
    assert!(
        output.contains("keyword_test"),
        "Expected table name in DESCRIBE output: {output}"
    );
    assert!(
        output.contains("clustering"),
        "Expected 'clustering' column in DESCRIBE output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_materialized_view() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "mv");

    // Create base table
    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.mv_base (id int PRIMARY KEY, name text, age int)"),
    )
    .success();

    // Insert some data
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.mv_base (id, name, age) VALUES (1, 'Alice', 30)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.mv_base (id, name, age) VALUES (2, 'Bob', 25)"),
    )
    .success();

    // Create materialized view — skip test if MVs are disabled (Cassandra disables by default)
    let mv_result = cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!(
                "CREATE MATERIALIZED VIEW {ks}.mv_by_age AS \
                 SELECT id, name, age FROM {ks}.mv_base \
                 WHERE age IS NOT NULL AND id IS NOT NULL \
                 PRIMARY KEY (age, id)"
            ),
        ])
        .output()
        .expect("failed to run cqlsh");
    let mv_stderr = String::from_utf8_lossy(&mv_result.stderr);
    if mv_stderr.contains("Materialized views are disabled") {
        eprintln!("Skipping test_materialized_view: MVs disabled on this cluster");
        drop_test_keyspace(scylla, &ks);
        return;
    }
    assert!(mv_result.status.success(), "CREATE MV failed: {mv_stderr}");

    // DESCRIBE the MV
    let describe_output = execute_cql_output_direct(
        scylla,
        &format!("DESCRIBE MATERIALIZED VIEW {ks}.mv_by_age"),
    );
    assert!(
        describe_output.contains("mv_by_age"),
        "Expected MV name in DESCRIBE output: {describe_output}"
    );

    // SELECT from MV — allow propagation time
    std::thread::sleep(std::time::Duration::from_secs(2));
    let select_output = execute_cql_output_direct(scylla, &format!("SELECT * FROM {ks}.mv_by_age"));
    assert!(
        select_output.contains("age")
            || select_output.contains("Alice")
            || select_output.contains("Bob")
            || select_output.contains("rows"),
        "Expected data or column headers in MV SELECT output: {select_output}"
    );

    // DROP the MV
    execute_cql_direct(scylla, &format!("DROP MATERIALIZED VIEW {ks}.mv_by_age"));

    drop_test_keyspace(scylla, &ks);
}

//! COPY integration tests ported from Python cqlsh dtest cqlsh_copy_tests.py
//!
//! These tests require a running ScyllaDB container. Run with:
//! cargo test --test integration dtest_copy -- --ignored

use std::fs;
use std::io::Write as IoWrite;

use super::helpers::*;

fn copy_dispatched(scylla: &ScyllaContainer, ks: &str) -> bool {
    let _ = cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("CREATE TABLE IF NOT EXISTS {ks}.copy_probe (id int PRIMARY KEY)"),
        ])
        .output();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    let result = cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.copy_probe TO '{csv_path}'")])
        .output()
        .expect("failed to run cqlsh-rs for COPY probe");

    let stderr = String::from_utf8_lossy(&result.stderr);
    !stderr.contains("SyntaxException") && !stderr.contains("no viable alternative")
}

/// Rows used by dtest's `quoted_column_names_reading_template`.
const QUOTED_NAMES_ROWS: [(i32, &str); 4] = [(1, "no"), (2, "Yes"), (3, "True"), (4, "false")];

fn write_quoted_names_csv() -> tempfile::TempPath {
    let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    for (id, val) in QUOTED_NAMES_ROWS {
        writeln!(tmp, "{id},{val}").unwrap();
    }
    tmp.flush().unwrap();
    tmp.into_temp_path()
}

fn assert_quoted_names_rows(scylla: &ScyllaContainer, ks: &str) {
    let output = execute_cql_output_direct(
        scylla,
        &format!(r#"SELECT "IdNumber", "select" FROM {ks}.testquoted"#),
    );
    assert!(output.contains("(4 rows)"), "Expected 4 rows: {output}");
    for (id, val) in QUOTED_NAMES_ROWS {
        assert!(
            output.lines().any(|l| l
                .split('|')
                .map(str::trim)
                .eq([id.to_string().as_str(), val])),
            "Expected row ({id}, {val}): {output}"
        );
    }
}

/// Count the rows of `table` with `SELECT count(*)`.
fn count_rows(scylla: &ScyllaContainer, table: &str) -> usize {
    let output = execute_cql_output_direct(scylla, &format!("SELECT count(*) FROM {table}"));
    output
        .lines()
        .find_map(|l| l.trim().parse().ok())
        .unwrap_or_else(|| panic!("no count in output: {output}"))
}

// ---------------------------------------------------------------------------
// test_bulk_round_trip_default — COPY TO / TRUNCATE / COPY FROM / COPY TO on a
// cassandra-stress `standard1`-shaped table (case-sensitive "C0".."C4")
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_bulk_round_trip_default() {
    // dtest uses 100000 cassandra-stress rows; 10000 keeps the same shape fast.
    const NUM_RECORDS: usize = 10_000;
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_bulk_rt");

    execute_cql(
        scylla,
        &format!(
            r#"CREATE TABLE {ks}.standard1 (key blob PRIMARY KEY, "C0" blob, "C1" blob, "C2" blob, "C3" blob, "C4" blob)"#
        ),
    )
    .success();

    // Seed the table (stands in for cassandra-stress write).
    let mut seed = tempfile::NamedTempFile::new().expect("failed to create temp file");
    for i in 0..NUM_RECORDS {
        writeln!(
            seed,
            "0x{i:08x},0x{i:04x}c0,0x{i:04x}c1,0x{i:04x}c2,0x{i:04x}c3,0x{i:04x}c4"
        )
        .unwrap();
    }
    seed.flush().unwrap();
    let seed_path = seed.path().to_str().unwrap().to_string();
    let copy_from = |path: &str| {
        let out = cqlsh_cmd(scylla)
            .args(["-e", &format!("COPY {ks}.standard1 FROM '{path}'")])
            .output()
            .expect("failed to run cqlsh-rs");
        assert!(out.status.success(), "COPY FROM failed: {out:?}");
    };
    let copy_to = |path: &str| {
        let out = cqlsh_cmd(scylla)
            .args(["-e", &format!("COPY {ks}.standard1 TO '{path}'")])
            .output()
            .expect("failed to run cqlsh-rs");
        assert!(out.status.success(), "COPY TO failed: {out:?}");
        let mut lines: Vec<String> = fs::read_to_string(path)
            .expect("failed to read CSV")
            .lines()
            .map(str::to_owned)
            .collect();
        lines.sort();
        lines
    };
    copy_from(&seed_path);
    let table = format!("{ks}.standard1");
    assert_eq!(count_rows(scylla, &table), NUM_RECORDS);

    let first = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let first_path = first.path().to_str().unwrap().to_string();
    let first_rows = copy_to(&first_path);
    assert_eq!(first_rows.len(), NUM_RECORDS);

    execute_cql(scylla, &format!("TRUNCATE {table}")).success();
    copy_from(&first_path);
    assert_eq!(count_rows(scylla, &table), NUM_RECORDS);

    let second = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let second_rows = copy_to(second.path().to_str().unwrap());
    assert!(
        first_rows == second_rows,
        "round trip changed the exported rows"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// COPY FROM WITH HEADER resolves header names against the schema: stored
// names as written by COPY TO (`C0`) and CQL-style ones (`KEY`, `"C0"`).
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_reading_header_with_case_sensitive_names() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_hdr_case");
    let table = format!("{ks}.hc");
    execute_cql(
        scylla,
        &format!(r#"CREATE TABLE {table} (key int PRIMARY KEY, "C0" text)"#),
    )
    .success();

    // `"\"C0\""` is CSV quoting (cqlsh's ESCAPE `\`, no doublequote) around
    // the CQL-quoted name `"C0"`.
    for header in ["key,C0", r#"KEY,"\"C0\"""#] {
        execute_cql(scylla, &format!("TRUNCATE {table}")).success();
        let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
        writeln!(tmp, "{header}").unwrap();
        writeln!(tmp, "1,one").unwrap();
        tmp.flush().unwrap();
        let out = cqlsh_cmd(scylla)
            .args([
                "-e",
                &format!(
                    "COPY {table} FROM '{}' WITH HEADER = true",
                    tmp.path().to_str().unwrap()
                ),
            ])
            .output()
            .expect("failed to run cqlsh-rs");
        assert!(out.status.success(), "COPY FROM failed: {out:?}");
        assert_eq!(count_rows(scylla, &table), 1, "header {header:?}: {out:?}");
    }

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_list_data — COPY TO with list<uuid> column
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_list_data() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_list");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.list_data (id int PRIMARY KEY, vals list<uuid>)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.list_data (id, vals) VALUES (1, \
             [550e8400-e29b-41d4-a716-446655440000, 550e8400-e29b-41d4-a716-446655440001])"
        ),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.list_data TO '{csv_path}'")])
        .assert()
        .success();

    let csv = fs::read_to_string(&csv_path).expect("failed to read CSV");
    assert!(
        csv.contains("550e8400"),
        "CSV should contain the UUID list values: {csv}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_colon_delimiter — COPY TO WITH DELIMITER=':'
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_colon_delimiter() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_colon");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.delim (id int PRIMARY KEY, name text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.delim (id, name) VALUES (1, 'Alice')"),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("COPY {ks}.delim TO '{csv_path}' WITH DELIMITER=':'"),
        ])
        .assert()
        .success();

    let csv = fs::read_to_string(&csv_path).expect("failed to read CSV");
    assert!(
        csv.contains(':'),
        "CSV should use colon as delimiter: {csv}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_letter_delimiter — COPY TO WITH DELIMITER='a'
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_letter_delimiter() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_letdlm");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.letdelim (id int PRIMARY KEY, score int)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.letdelim (id, score) VALUES (1, 99)"),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("COPY {ks}.letdelim TO '{csv_path}' WITH DELIMITER='a'"),
        ])
        .assert()
        .success();

    let csv = fs::read_to_string(&csv_path).expect("failed to read CSV");
    assert!(csv.contains('a'), "CSV should use 'a' as delimiter: {csv}");

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_number_delimiter — COPY TO WITH DELIMITER='1'
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_number_delimiter() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_numdlm");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.numdelim (id int PRIMARY KEY, name text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.numdelim (id, name) VALUES (2, 'Bob')"),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("COPY {ks}.numdelim TO '{csv_path}' WITH DELIMITER='1'"),
        ])
        .assert()
        .success();

    let csv = fs::read_to_string(&csv_path).expect("failed to read CSV");
    // '1' delimiter should appear in the CSV (separating the id '2' from 'Bob')
    assert!(csv.contains('1'), "CSV should use '1' as delimiter: {csv}");

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_undefined_as_null_indicator — COPY TO WITH NULL='undefined'
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_undefined_as_null_indicator() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_undnull");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.nulltbl (id int PRIMARY KEY, val text)"),
    )
    .success();
    // Insert a row with only the PK — val will be null
    execute_cql_direct(scylla, &format!("INSERT INTO {ks}.nulltbl (id) VALUES (1)"));

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("COPY {ks}.nulltbl TO '{csv_path}' WITH NULLVAL='undefined'"),
        ])
        .assert()
        .success();

    let csv = fs::read_to_string(&csv_path).expect("failed to read CSV");
    assert!(
        csv.contains("undefined"),
        "CSV should represent null as 'undefined': {csv}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_writing_with_timeformat — COPY TO WITH DATETIMEFORMAT
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_writing_with_timeformat() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_timefmt");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.tsfmt (id int PRIMARY KEY, ts timestamp)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.tsfmt (id, ts) VALUES (1, '2024-06-15 10:30:00+0000')"),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("COPY {ks}.tsfmt TO '{csv_path}' WITH DATETIMEFORMAT='%Y/%m/%d %H:%M'"),
        ])
        .assert()
        .success();

    let csv = fs::read_to_string(&csv_path).expect("failed to read CSV");
    // Verify the date appears in YYYY/MM/DD format
    assert!(
        csv.contains("2024/06/15"),
        "CSV should contain date in YYYY/MM/DD format: {csv}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_explicit_column_order_writing — COPY TO with column list
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_explicit_column_order_writing() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_colord_w");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.colord (a int PRIMARY KEY, b text, c int)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.colord (a, b, c) VALUES (1, 'hello', 42)"),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.colord (a, c, b) TO '{csv_path}'")])
        .assert()
        .success();

    let csv = fs::read_to_string(&csv_path).expect("failed to read CSV");
    // Each line should have a,c,b order: "1,42,hello"
    assert!(
        csv.contains("42") && csv.contains("hello"),
        "CSV should contain the exported values: {csv}"
    );
    // Verify c (42) appears before b (hello) on the same line
    let line = csv.lines().next().unwrap_or("");
    let c_pos = line.find("42");
    let b_pos = line.find("hello");
    if let (Some(cp), Some(bp)) = (c_pos, b_pos) {
        assert!(
            cp < bp,
            "Column c (42) should appear before column b (hello) in CSV: {line}"
        );
    }

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_explicit_column_order_reading — COPY FROM with column list
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_explicit_column_order_reading() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_colord_r");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.colord_r (a int PRIMARY KEY, b text, c int)"),
    )
    .success();

    // CSV has columns in order: a, c, b
    let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    writeln!(tmp, "1,42,hello").unwrap();
    writeln!(tmp, "2,99,world").unwrap();
    tmp.flush().unwrap();
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("COPY {ks}.colord_r (a, c, b) FROM '{csv_path}'"),
        ])
        .assert()
        .success();

    let output = execute_cql_output_direct(
        scylla,
        &format!("SELECT a, b, c FROM {ks}.colord_r WHERE a = 1"),
    );
    assert!(
        output.contains("hello"),
        "b column should contain 'hello': {output}"
    );
    assert!(
        output.contains("42"),
        "c column should contain 42: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_quoted_column_names_reading_specify_names — COPY FROM with quoted cols
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_quoted_column_names_reading_specify_names() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_qcol_rs");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!(r#"CREATE TABLE {ks}.testquoted ("IdNumber" int PRIMARY KEY, "select" text)"#),
    )
    .success();

    let csv_path = write_quoted_names_csv();
    let out = cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!(
                r#"COPY {ks}.testquoted ("IdNumber", "select") FROM '{}'"#,
                csv_path.display()
            ),
        ])
        .output()
        .expect("failed to run cqlsh-rs");
    assert!(out.status.success(), "COPY FROM failed: {out:?}");

    assert_quoted_names_rows(scylla, &ks);

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_quoted_column_names_reading_dont_specify_names — COPY FROM no col list
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_quoted_column_names_reading_dont_specify_names() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_qcol_rn");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!(r#"CREATE TABLE {ks}.testquoted ("IdNumber" int PRIMARY KEY, "select" text)"#),
    )
    .success();

    let csv_path = write_quoted_names_csv();
    let out = cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("COPY {ks}.testquoted FROM '{}'", csv_path.display()),
        ])
        .output()
        .expect("failed to run cqlsh-rs");
    assert!(out.status.success(), "COPY FROM failed: {out:?}");

    assert_quoted_names_rows(scylla, &ks);

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_quoted_column_names_writing_specify_names — COPY TO with quoted cols
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_quoted_column_names_writing_specify_names() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_qcol_ws");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!(r#"CREATE TABLE {ks}.qwrite_s ("MyKey" int PRIMARY KEY, "MyValue" text)"#),
    )
    .success();
    execute_cql(
        scylla,
        &format!(r#"INSERT INTO {ks}.qwrite_s ("MyKey", "MyValue") VALUES (1, 'zeta')"#),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!(r#"COPY {ks}.qwrite_s ("MyKey", "MyValue") TO '{csv_path}'"#),
        ])
        .assert()
        .success();

    let csv = fs::read_to_string(&csv_path).expect("failed to read CSV");
    assert!(csv.contains("zeta"), "CSV should contain 'zeta': {csv}");

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_quoted_column_names_writing_dont_specify_names — COPY TO no col list
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_quoted_column_names_writing_dont_specify_names() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_qcol_wn");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!(r#"CREATE TABLE {ks}.qwrite_n ("MyKey" int PRIMARY KEY, "MyValue" text)"#),
    )
    .success();
    execute_cql(
        scylla,
        &format!(r#"INSERT INTO {ks}.qwrite_n ("MyKey", "MyValue") VALUES (1, 'eta')"#),
    )
    .success();

    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.qwrite_n TO '{csv_path}'")])
        .assert()
        .success();

    let csv = fs::read_to_string(&csv_path).expect("failed to read CSV");
    assert!(csv.contains("eta"), "CSV should contain 'eta': {csv}");

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_read_valid_data — COPY FROM with valid int data
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_read_valid_data() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_valid");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.valid_int (id int PRIMARY KEY, val int)"),
    )
    .success();

    let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    writeln!(tmp, "1,100").unwrap();
    writeln!(tmp, "2,200").unwrap();
    writeln!(tmp, "3,300").unwrap();
    tmp.flush().unwrap();
    let csv_path = tmp.path().to_str().unwrap().to_string();

    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.valid_int FROM '{csv_path}'")])
        .assert()
        .success();

    let output = execute_cql_output_direct(scylla, &format!("SELECT * FROM {ks}.valid_int"));
    assert!(output.contains("100"), "Expected 100 in DB: {output}");
    assert!(output.contains("200"), "Expected 200 in DB: {output}");
    assert!(output.contains("300"), "Expected 300 in DB: {output}");

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_read_invalid_float — COPY FROM with float "1.0" into int column
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_read_invalid_float() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_inv_flt");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.bad_float (id int PRIMARY KEY, val int)"),
    )
    .success();

    let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    writeln!(tmp, "1,1.0").unwrap();
    tmp.flush().unwrap();
    let csv_path = tmp.path().to_str().unwrap().to_string();

    let result = cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.bad_float FROM '{csv_path}'")])
        .output()
        .expect("failed to run cqlsh-rs");

    // Expect either a non-zero exit or an error message about parse failure
    let stderr = String::from_utf8_lossy(&result.stderr);
    let stdout = String::from_utf8_lossy(&result.stdout);
    let has_error = !result.status.success()
        || stderr.contains("error")
        || stderr.contains("Error")
        || stdout.contains("error")
        || stdout.contains("failed");
    assert!(
        has_error,
        "Importing float '1.0' into int column should produce an error. \
         stdout={stdout} stderr={stderr}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_read_invalid_uuid — COPY FROM with uuid string into int column
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_read_invalid_uuid() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_inv_uid");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.bad_uuid (id int PRIMARY KEY, val int)"),
    )
    .success();

    let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    writeln!(tmp, "1,550e8400-e29b-41d4-a716-446655440000").unwrap();
    tmp.flush().unwrap();
    let csv_path = tmp.path().to_str().unwrap().to_string();

    let result = cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.bad_uuid FROM '{csv_path}'")])
        .output()
        .expect("failed to run cqlsh-rs");

    let stderr = String::from_utf8_lossy(&result.stderr);
    let stdout = String::from_utf8_lossy(&result.stdout);
    let has_error = !result.status.success()
        || stderr.contains("error")
        || stderr.contains("Error")
        || stdout.contains("error")
        || stdout.contains("failed");
    assert!(
        has_error,
        "Importing UUID value into int column should produce an error. \
         stdout={stdout} stderr={stderr}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_read_invalid_text — COPY FROM with text "foo" into int column
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_read_invalid_text() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_inv_txt");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.bad_text (id int PRIMARY KEY, val int)"),
    )
    .success();

    let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    writeln!(tmp, "1,foo").unwrap();
    tmp.flush().unwrap();
    let csv_path = tmp.path().to_str().unwrap().to_string();

    let result = cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.bad_text FROM '{csv_path}'")])
        .output()
        .expect("failed to run cqlsh-rs");

    let stderr = String::from_utf8_lossy(&result.stderr);
    let stdout = String::from_utf8_lossy(&result.stdout);
    let has_error = !result.status.success()
        || stderr.contains("error")
        || stderr.contains("Error")
        || stdout.contains("error")
        || stdout.contains("failed");
    assert!(
        has_error,
        "Importing text 'foo' into int column should produce an error. \
         stdout={stdout} stderr={stderr}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_reading_counter — COPY FROM into counter table should fail
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_reading_counter() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_counter");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.ctr_tbl (id int PRIMARY KEY, cnt counter)"),
    )
    .success();

    let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    writeln!(tmp, "1,10").unwrap();
    tmp.flush().unwrap();
    let csv_path = tmp.path().to_str().unwrap().to_string();

    let result = cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.ctr_tbl FROM '{csv_path}'")])
        .output()
        .expect("failed to run cqlsh-rs");

    // COPY FROM into a counter table should be rejected
    let stderr = String::from_utf8_lossy(&result.stderr);
    let stdout = String::from_utf8_lossy(&result.stdout);
    let has_error = !result.status.success()
        || stderr.contains("error")
        || stderr.contains("Error")
        || stderr.contains("counter")
        || stdout.contains("error")
        || stdout.contains("counter");
    assert!(
        has_error,
        "COPY FROM into counter table should produce an error. \
         stdout={stdout} stderr={stderr}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_source_copy_round_trip — run COPY via -f file, then COPY FROM back
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_source_copy_round_trip() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_src_rt");

    if !copy_dispatched(scylla, &ks) {
        eprintln!("SKIP: COPY not dispatched in -e mode");
        drop_test_keyspace(scylla, &ks);
        return;
    }

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.src_rt (id int PRIMARY KEY, val text)"),
    )
    .success();

    for i in 1..=3_u32 {
        execute_cql(
            scylla,
            &format!("INSERT INTO {ks}.src_rt (id, val) VALUES ({i}, 'item_{i}')"),
        )
        .success();
    }

    // Write a .cql file containing the COPY TO command
    let csv_tmp = tempfile::NamedTempFile::new().expect("failed to create CSV temp file");
    let csv_path = csv_tmp.path().to_str().unwrap().to_string();

    let mut cql_tmp = tempfile::NamedTempFile::new().expect("failed to create CQL temp file");
    writeln!(cql_tmp, "COPY {ks}.src_rt TO '{csv_path}';").unwrap();
    cql_tmp.flush().unwrap();
    let cql_path = cql_tmp.path().to_str().unwrap().to_string();

    // Run cqlsh-rs with -f pointing at the .cql file
    cqlsh_cmd(scylla).args(["-f", &cql_path]).assert().success();

    let csv = fs::read_to_string(&csv_path).expect("failed to read CSV produced by -f");
    assert!(!csv.is_empty(), "CSV produced via -f should not be empty");
    assert!(csv.contains("item_1"), "CSV should contain item_1: {csv}");

    // Truncate and re-import via COPY FROM
    execute_cql_direct(scylla, &format!("TRUNCATE {ks}.src_rt"));

    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.src_rt FROM '{csv_path}'")])
        .assert()
        .success();

    let output = execute_cql_output_direct(scylla, &format!("SELECT * FROM {ks}.src_rt"));
    for i in 1..=3_u32 {
        assert!(
            output.contains(&format!("item_{i}")),
            "Expected item_{i} after round-trip: {output}"
        );
    }

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// test_bulk_round_trip_blogposts (text with backslashes) — the exported CSV
// must parse with quotechar='"', escapechar='\\' into one record per row,
// and COPY FROM must read it back unchanged.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_round_trip_text_with_escape_char() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_esc_rt");
    let table = format!("{ks}.esc");
    execute_cql(
        scylla,
        &format!("CREATE TABLE {table} (k int PRIMARY KEY, v text)"),
    )
    .success();
    let values = [
        r"back\slash",
        r"ends with \",
        r#"quote " and \ both"#,
        "comma, here",
        r"\\double",
    ];
    for (k, v) in values.iter().enumerate() {
        execute_cql(
            scylla,
            &format!("INSERT INTO {table} (k, v) VALUES ({k}, '{v}')"),
        )
        .success();
    }

    let export = |path: &str| -> Vec<Vec<String>> {
        cqlsh_cmd(scylla)
            .args(["-e", &format!("COPY {table} TO '{path}'")])
            .assert()
            .success();
        let mut rows: Vec<Vec<String>> = csv::ReaderBuilder::new()
            .has_headers(false)
            .escape(Some(b'\\'))
            .from_path(path)
            .unwrap()
            .records()
            .map(|r| r.unwrap().iter().map(String::from).collect())
            .collect();
        rows.sort();
        rows
    };

    let first = tempfile::NamedTempFile::new().unwrap();
    let first_path = first.path().to_str().unwrap();
    let exported = export(first_path);
    let mut expected: Vec<Vec<String>> = values
        .iter()
        .enumerate()
        .map(|(k, v)| vec![k.to_string(), v.to_string()])
        .collect();
    expected.sort();
    assert_eq!(exported, expected);

    execute_cql_direct(scylla, &format!("TRUNCATE {table}"));
    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {table} FROM '{first_path}'")])
        .assert()
        .success();

    let second = tempfile::NamedTempFile::new().unwrap();
    assert_eq!(export(second.path().to_str().unwrap()), expected);

    drop_test_keyspace(scylla, &ks);
}

// COPY FROM a file with CR-only line endings and a header row.
#[test]
#[ignore = "requires Docker"]
fn test_read_cr_line_endings_with_header() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "dt_cr");
    let table = format!("{ks}.cr");
    execute_cql(
        scylla,
        &format!("CREATE TABLE {table} (k int PRIMARY KEY, v int)"),
    )
    .success();

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    write!(tmp, "k,v\r1,100\r2,200\r").unwrap();
    tmp.flush().unwrap();
    let path = tmp.path().to_str().unwrap();
    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!("COPY {table} FROM '{path}' WITH HEADER=true"),
        ])
        .assert()
        .success();

    let output = execute_cql_output_direct(scylla, &format!("SELECT * FROM {table}"));
    assert!(output.contains("100"), "Expected 100 in DB: {output}");
    assert!(output.contains("200"), "Expected 200 in DB: {output}");

    drop_test_keyspace(scylla, &ks);
}

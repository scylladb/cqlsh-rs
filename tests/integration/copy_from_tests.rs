//! Integration tests for COPY FROM command.
//!
//! These tests require a running ScyllaDB container. Run with:
//! cargo test --test integration copy_from -- --ignored

use super::helpers;

/// Basic CSV import: all rows imported, count reported.
#[test]
#[ignore = "requires ScyllaDB container"]
fn copy_from_basic_csv() {
    let scylla = helpers::get_scylla();
    let _ks = helpers::create_test_keyspace(scylla, "copy_basic");
    // TODO: create table, write temp CSV, run COPY FROM, verify row count
}

/// COPY FROM WITH HEADER=true reads column names from first CSV row.
#[test]
#[ignore = "requires ScyllaDB container"]
fn copy_from_with_header() {
    let scylla = helpers::get_scylla();
    let _ks = helpers::create_test_keyspace(scylla, "copy_header");
    // TODO: create table, write CSV with header row, run COPY FROM WITH HEADER=true
}

/// All 25 CQL scalar types are parsed and inserted correctly.
#[test]
#[ignore = "requires ScyllaDB container"]
fn copy_from_all_scalar_types() {
    let scylla = helpers::get_scylla();
    let _ks = helpers::create_test_keyspace(scylla, "copy_types");
    // TODO: create table covering ascii, text, int, bigint, smallint, tinyint, float,
    //       double, boolean, uuid, timeuuid, timestamp, date, time, inet, blob,
    //       varint, decimal; write CSV row; verify via SELECT
}

/// Null handling: empty fields and custom NULLVAL are stored as CQL null.
#[test]
#[ignore = "requires ScyllaDB container"]
fn copy_from_null_handling() {
    let scylla = helpers::get_scylla();
    let _ks = helpers::create_test_keyspace(scylla, "copy_null");
    // TODO: table with nullable columns, CSV with empty fields and "NULL" strings,
    //       verify nulls via SELECT
}

/// TTL option inserts rows with the specified TTL.
#[test]
#[ignore = "requires ScyllaDB container"]
fn copy_from_ttl_option() {
    let scylla = helpers::get_scylla();
    let _ks = helpers::create_test_keyspace(scylla, "copy_ttl");
    // TODO: COPY FROM WITH TTL=1, verify row exists immediately, sleep 2s, verify gone
}

/// Round-trip: COPY TO produces a CSV that COPY FROM can re-import cleanly.
#[test]
#[ignore = "requires ScyllaDB container"]
fn copy_from_roundtrip_with_copy_to() {
    let scylla = helpers::get_scylla();
    let _ks = helpers::create_test_keyspace(scylla, "copy_roundtrip");
    // TODO: insert rows, COPY TO temp file, truncate table, COPY FROM temp file,
    //       compare original rows with re-imported rows
}

/// MAXPARSEERRORS stops import after N parse failures.
#[test]
#[ignore = "requires ScyllaDB container"]
fn copy_from_max_parse_errors() {
    let scylla = helpers::get_scylla();
    let _ks = helpers::create_test_keyspace(scylla, "copy_parseerr");
    // TODO: CSV with >N invalid rows, COPY FROM WITH MAXPARSEERRORS=N,
    //       verify error message and partial import count
}

/// ERRFILE writes failed rows to a file.
#[test]
#[ignore = "requires ScyllaDB container"]
fn copy_from_errfile() {
    let scylla = helpers::get_scylla();
    let _ks = helpers::create_test_keyspace(scylla, "copy_errfile");
    // TODO: CSV with some invalid rows, COPY FROM WITH ERRFILE=/tmp/errs.csv,
    //       verify failed rows appear in errfile
}

/// PREPAREDSTATEMENTS=false uses string INSERT path.
#[test]
#[ignore = "requires ScyllaDB container"]
fn copy_from_prepared_vs_unprepared() {
    let scylla = helpers::get_scylla();
    let _ks = helpers::create_test_keyspace(scylla, "copy_prepstmt");
    // TODO: run same import with PREPAREDSTATEMENTS=true and PREPAREDSTATEMENTS=false,
    //       verify identical row counts and data
}

/// NUMPROCESSES > 1 imports correctly with parallel workers.
#[test]
#[ignore = "requires ScyllaDB container"]
fn copy_from_numprocesses_parallel() {
    let scylla = helpers::get_scylla();
    let _ks = helpers::create_test_keyspace(scylla, "copy_parallel");
    // TODO: large CSV, COPY FROM WITH NUMPROCESSES=4, verify all rows imported
}

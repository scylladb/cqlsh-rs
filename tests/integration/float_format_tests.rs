//! float/double formatting, ported from scylla-dtest (issue #194):
//! - `cql_cast_test.py::TestCQLCast` — casts/aggregates to float and double
//! - `cqlsh_tests.py::TestCqlsh::test_float_formatting`
//! - `cqlsh_tests.py::TestCqlsh::test_copy_to`
//!
//! Python cqlsh prints floats with `float_precision` (default 5) significant
//! digits plus the integer digits, via `%g`: no trailing zeros, scientific
//! notation for large/small exponents.

use std::fs;
use std::io::Write as IoWrite;

use super::helpers::*;

/// The value of a single-row, single-column SELECT, i.e. line 3 of the
/// output (blank, header, separator, value) — what the dtest compares.
fn single_value(output: &str) -> String {
    output
        .lines()
        .nth(3)
        .unwrap_or_else(|| panic!("no value line in output: {output:?}"))
        .trim()
        .to_string()
}

/// The lines of a CSV file, sorted (COPY TO row order is token order).
fn sorted_lines(path: &str) -> Vec<String> {
    let mut lines: Vec<String> = fs::read_to_string(path)
        .expect("read csv")
        .lines()
        .map(String::from)
        .collect();
    lines.sort();
    lines
}

// ---------------------------------------------------------------------------
// cql_cast_test.py — CAST / aggregates to float and double
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_cast_to_float_and_double() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "cast_fmt");

    // (column type, values) — the same data as TestData.POSITIVE_VALUES.
    let tables: &[(&str, &[&str])] = &[
        ("bigint", &["7657139128428828644", "125", "-5698"]),
        ("decimal", &["76571.345", "-254.256987"]),
        ("double", &["76571.345", "-254.256987"]),
        (
            "float",
            &["-564", "3543.4565429688", "123.25993624886543132316548678"],
        ),
        ("int", &["0", "32767", "-32767", "-12876", "2654"]),
        ("smallint", &["32767", "-32767", "-12876", "2654"]),
        ("tinyint", &["127", "-128", "35"]),
        (
            "varint",
            &[
                "32767456456456456456545678943512357658768763546575675",
                "-32767456456456456456545678943512357658768769876743674",
                "4258",
            ],
        ),
    ];
    for (ty, values) in tables {
        execute_cql_direct(
            scylla,
            &format!("CREATE TABLE {ks}.cast_{ty} (k int PRIMARY KEY, v {ty})"),
        );
        for (k, v) in values.iter().enumerate() {
            execute_cql_direct(
                scylla,
                &format!("INSERT INTO {ks}.cast_{ty} (k, v) VALUES ({k}, {v})"),
            );
        }
    }

    // (query against cast_<type>, expected) — expectations copied from the dtest.
    let cases: &[(&str, &str, &str)] = &[
        ("bigint", "cast(v as float) ... WHERE k=0", "7.6571e+18"),
        ("bigint", "cast(v as double) ... WHERE k=0", "7.6571e+18"),
        ("bigint", "cast(v as float) ... WHERE k=1", "125"),
        ("bigint", "avg(cast(v as float))", "2.5524e+18"),
        ("bigint", "avg(cast(v as double))", "2.5524e+18"),
        ("bigint", "sum(cast(v as double))", "7.6571e+18"),
        ("bigint", "min(cast(v as float))", "-5698"),
        ("decimal", "cast(v as float) ... WHERE k=0", "76571.34375"),
        ("decimal", "cast(v as double) ... WHERE k=0", "76571.345"),
        ("decimal", "cast(v as double) ... WHERE k=1", "-254.25699"),
        ("decimal", "avg(cast(v as float))", "38158.54297"),
        ("decimal", "avg(cast(v as double))", "38158.54401"),
        ("decimal", "sum(cast(v as float))", "76317.08594"),
        ("decimal", "cast(sum(v) as double)", "76317.08801"),
        ("decimal", "cast(min(v) as float)", "-254.25699"),
        ("double", "cast(v as float) ... WHERE k=0", "76571.34375"),
        ("double", "cast(v as float) ... WHERE k=1", "-254.25699"),
        ("double", "avg(cast(v as float))", "38158.54297"),
        ("double", "sum(cast(v as float))", "76317.08594"),
        ("float", "cast(v as double) ... WHERE k=0", "-564"),
        ("float", "cast(v as double) ... WHERE k=1", "3543.45654"),
        ("float", "cast(v as double) ... WHERE k=2", "123.25993"),
        ("float", "avg(cast(v as double))", "1034.23883"),
        ("float", "sum(cast(v as double))", "3102.71648"),
        ("float", "max(cast(v as double))", "3543.45654"),
        ("int", "avg(cast(v as float))", "-2044.40002"),
        ("int", "avg(cast(v as double))", "-2044.4"),
        ("int", "sum(cast(v as float))", "-10222"),
        ("smallint", "avg(cast(v as float))", "-2555.5"),
        ("tinyint", "avg(cast(v as float))", "11.33333"),
        ("tinyint", "avg(cast(v as double))", "11.33333"),
        ("tinyint", "cast(v as float) ... WHERE k=1", "-128"),
        ("varint", "cast(v as float) ... WHERE k=0", "Infinity"),
        ("varint", "cast(v as double) ... WHERE k=0", "3.2767e+52"),
        ("varint", "cast(v as float) ... WHERE k=1", "-Infinity"),
        ("varint", "cast(v as double) ... WHERE k=1", "-3.2767e+52"),
        ("varint", "avg(cast(v as float))", "NaN"),
        ("varint", "cast(avg(v) as float)", "-2.1101e+09"),
        ("varint", "cast(sum(v) as double)", "-6.3302e+09"),
    ];

    let mut failures = Vec::new();
    for (ty, expr, expected) in cases {
        let (select, filter) = expr.split_once(" ... ").unwrap_or((expr, ""));
        let query = format!("SELECT {select} FROM {ks}.cast_{ty} {filter}");
        let actual = single_value(&execute_cql_output_direct(scylla, &query));
        if actual != *expected {
            failures.push(format!("{query}: expected {expected}, got {actual}"));
        }
    }

    // test_cast_udt: a float field cast to double.
    execute_cql_direct(scylla, &format!("CREATE TYPE {ks}.u (f float)"));
    execute_cql_direct(
        scylla,
        &format!("CREATE TABLE {ks}.cast_udt (k int PRIMARY KEY, v frozen<u>)"),
    );
    execute_cql_direct(
        scylla,
        &format!("INSERT INTO {ks}.cast_udt (k, v) VALUES (0, {{f: 55646.234556984}})"),
    );
    let actual = single_value(&execute_cql_output_direct(
        scylla,
        &format!("SELECT cast(v.f as double) FROM {ks}.cast_udt WHERE k=0"),
    ));
    if actual != "55646.23438" {
        failures.push(format!("udt cast: expected 55646.23438, got {actual}"));
    }

    drop_test_keyspace(scylla, &ks);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// ---------------------------------------------------------------------------
// cqlsh_tests.py::test_float_formatting
// ---------------------------------------------------------------------------

const FLOAT_INPUTS: [&str; 33] = [
    "0.00000006",
    "0.0000006",
    "0.000006",
    "0.00006",
    "0.0006",
    "0.006",
    "0.06",
    "0.6",
    "6",
    "6.0",
    "6.00",
    "6.000",
    "6.00000",
    "6.000000",
    "6.1",
    "6.12",
    "6.123",
    "6.1234",
    "6.12345",
    "6.123454",
    "6.123455",
    "6.123456",
    "6.1234565",
    "6.1234555",
    "6.12345555",
    "6.12345555555555",
    "16.12345",
    "116.12345",
    "1116.12345",
    "11116.12345",
    "111116.12345",
    "1111116.12345",
    "11111116.12345",
];

/// (val1 double, val2 float) as printed by Python cqlsh for the '+' rows.
const FLOAT_EXPECTED: [(&str, &str); 33] = [
    ("6e-08", "6e-08"),
    ("6e-07", "6e-07"),
    ("6e-06", "6e-06"),
    ("6e-05", "6e-05"),
    ("0.0006", "0.0006"),
    ("0.006", "0.006"),
    ("0.06", "0.06"),
    ("0.6", "0.6"),
    ("6", "6"),
    ("6", "6"),
    ("6", "6"),
    ("6", "6"),
    ("6", "6"),
    ("6", "6"),
    ("6.1", "6.1"),
    ("6.12", "6.12"),
    ("6.123", "6.123"),
    ("6.1234", "6.1234"),
    ("6.12345", "6.12345"),
    ("6.12345", "6.12345"),
    ("6.12345", "6.12346"),
    ("6.12346", "6.12346"),
    ("6.12346", "6.12346"),
    ("6.12346", "6.12346"),
    ("6.12346", "6.12346"),
    ("6.12346", "6.12346"),
    ("16.12345", "16.12345"),
    ("116.12345", "116.12345"),
    ("1116.12345", "1116.12341"),
    ("11116.12345", "11116.12305"),
    ("1.1112e+05", "1.1112e+05"),
    ("1.1111e+06", "1.1111e+06"),
    ("1.1111e+07", "1.1111e+07"),
];

#[test]
#[ignore = "requires Docker"]
fn test_float_formatting_dtest() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "fmt");
    execute_cql_direct(
        scylla,
        &format!(
            "CREATE TABLE {ks}.values (part text, id int, val1 double, val2 float, \
             PRIMARY KEY (part, id))"
        ),
    );
    for (sign, part) in [("", "+"), ("-", "-")] {
        for (i, v) in FLOAT_INPUTS.iter().enumerate() {
            execute_cql_direct(
                scylla,
                &format!(
                    "INSERT INTO {ks}.values (part, id, val1, val2) \
                     VALUES ('{part}', {}, {sign}{v}, {sign}{v})",
                    i + 1
                ),
            );
        }
    }

    for (sign, part) in [("", "+"), ("-", "-")] {
        let output = execute_cql_output_direct(
            scylla,
            &format!("SELECT id, val1, val2 FROM {ks}.values WHERE part = '{part}'"),
        );
        let rows: Vec<Vec<&str>> = output
            .lines()
            .skip(3)
            .filter(|l| l.contains('|'))
            .map(|l| l.split('|').map(str::trim).collect())
            .collect();
        assert_eq!(rows.len(), FLOAT_EXPECTED.len(), "{output}");
        for (i, (row, (d, f))) in rows.iter().zip(FLOAT_EXPECTED).enumerate() {
            assert_eq!(
                row,
                &vec![
                    (i + 1).to_string().as_str(),
                    &format!("{sign}{d}"),
                    &format!("{sign}{f}"),
                ],
                "part {part} row {}\n{output}",
                i + 1
            );
        }
    }

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// cqlshrc [ui] float_precision / double_precision
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_float_precision_from_cqlshrc() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "fmt_rc");
    execute_cql_direct(
        scylla,
        &format!("CREATE TABLE {ks}.t (k int PRIMARY KEY, f float, d double, l list<double>)"),
    );
    execute_cql_direct(
        scylla,
        &format!(
            "INSERT INTO {ks}.t (k, f, d, l) VALUES (0, 3.14159265, 3.14159265358979, \
             [3.14159265358979])"
        ),
    );

    let mut rc = tempfile::NamedTempFile::new().expect("temp cqlshrc");
    writeln!(rc, "[ui]\nfloat_precision = 3\ndouble_precision = 10").unwrap();
    let rc_path = rc.path().to_str().unwrap().to_string();

    let out = cqlsh_cmd(scylla)
        .args([
            "--cqlshrc",
            &rc_path,
            "-e",
            &format!("SELECT f, d, l FROM {ks}.t WHERE k = 0"),
        ])
        .output()
        .expect("run cqlsh-rs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = single_value(&stdout);
    let row: Vec<&str> = line.split('|').map(str::trim).collect();
    // Python cqlsh picks double_precision only for a top-level double column;
    // a double inside a collection uses float_precision.
    assert_eq!(row, ["3.142", "3.1415926536", "[3.142]"], "{stdout}");

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// cqlsh_tests.py::test_copy_to — float column written like Python's str()
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_copy_to_float_format() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "copy_fmt");
    execute_cql_direct(
        scylla,
        &format!(
            "CREATE TABLE {ks}.testcopyto (a int, b text, c float, d uuid, PRIMARY KEY (a, b))"
        ),
    );
    let ids = [0, 1, 7, 42, 123, 999, 4096, 9999];
    for i in ids {
        execute_cql_direct(
            scylla,
            &format!(
                "INSERT INTO {ks}.testcopyto (a, b, c, d) \
                 VALUES ({i}, '{i}', {i}.5, 11111111-1111-1111-1111-111111111111)"
            ),
        );
    }

    let tmp = tempfile::NamedTempFile::new().expect("temp csv");
    let csv_path = tmp.path().to_str().unwrap().to_string();
    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.testcopyto TO '{csv_path}'")])
        .assert()
        .success();

    let lines = sorted_lines(&csv_path);
    let mut expected: Vec<String> = ids
        .iter()
        .map(|i| format!("{i},{i},{i}.5,11111111-1111-1111-1111-111111111111"))
        .collect();
    expected.sort();
    assert_eq!(lines, expected);

    // FLOATPRECISION / DOUBLEPRECISION are significant digits too.
    // COPY TO writes the regular columns in schema order: k, d, f.
    execute_cql_direct(
        scylla,
        &format!("CREATE TABLE {ks}.prec (k int PRIMARY KEY, f float, d double)"),
    );
    execute_cql_direct(
        scylla,
        &format!("INSERT INTO {ks}.prec (k, f, d) VALUES (0, 76571.345, 7657139128428828644)"),
    );
    execute_cql_direct(
        scylla,
        &format!("INSERT INTO {ks}.prec (k, f, d) VALUES (1, 0.00000006, -254.256987)"),
    );
    cqlsh_cmd(scylla)
        .args(["-e", &format!("COPY {ks}.prec TO '{csv_path}'")])
        .assert()
        .success();
    let lines = sorted_lines(&csv_path);
    assert_eq!(
        lines,
        ["0,7.65713912843e+18,76571.34375", "1,-254.256987,6e-08"]
    );

    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!(
                "COPY {ks}.prec TO '{csv_path}' WITH FLOATPRECISION = 2 AND DOUBLEPRECISION = 3"
            ),
        ])
        .assert()
        .success();
    let lines = sorted_lines(&csv_path);
    assert_eq!(lines, ["0,7.66e+18,7.7e+04", "1,-254.257,6e-08"]);

    // Nested floats and doubles both get FLOATPRECISION, as in Python cqlsh.
    execute_cql_direct(
        scylla,
        &format!(
            "CREATE TABLE {ks}.nested (k int PRIMARY KEY, l list<double>, m map<text, float>)"
        ),
    );
    execute_cql_direct(
        scylla,
        &format!("INSERT INTO {ks}.nested (k, l, m) VALUES (0, [1.23456789], {{'a': 1.23456}})"),
    );
    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!(
                "COPY {ks}.nested TO '{csv_path}' WITH FLOATPRECISION = 3 AND DOUBLEPRECISION = 7"
            ),
        ])
        .assert()
        .success();
    assert_eq!(sorted_lines(&csv_path), ["0,[1.235],{'a': 1.235}"]);

    // ...and DECIMALSEP.
    cqlsh_cmd(scylla)
        .args([
            "-e",
            &format!(
                "COPY {ks}.nested TO '{csv_path}' WITH FLOATPRECISION = 3 AND DECIMALSEP = ',' AND DELIMITER = '|'"
            ),
        ])
        .assert()
        .success();
    assert_eq!(sorted_lines(&csv_path), ["0|[1,235]|{'a': 1,235}"]);

    drop_test_keyspace(scylla, &ks);
}

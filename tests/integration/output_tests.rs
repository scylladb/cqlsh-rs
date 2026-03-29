//! Output formatting integration tests for cqlsh-rs.
//!
//! Mirrors the Python cqlsh `test_cqlsh_output.py` test suite (Phase 3 of SP19).
//! Tests verify that output formatting, coloring, and display behavior match
//! Python cqlsh when run against a real ScyllaDB instance.

use super::helpers::*;

// ---------------------------------------------------------------------------
// 3.1 — No-color output (--no-color suppresses ANSI codes)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_no_color_output() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "nocolor_out");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.nc (id int PRIMARY KEY, name text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.nc (id, name) VALUES (1, 'Alice')"),
    )
    .success();

    let output = cqlsh_cmd(scylla)
        .args(["--no-color", "-e", &format!("SELECT * FROM {ks}.nc")])
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\x1b["),
        "ANSI escape codes found in --no-color output: {stdout}"
    );
    assert!(
        stdout.contains("Alice"),
        "Expected data in output: {stdout}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 3.2 — Color output (-C forces ANSI codes even when stdout is not a TTY)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_color_output_forced() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "color_out");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.co (id int PRIMARY KEY, name text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.co (id, name) VALUES (1, 'Bob')"),
    )
    .success();

    let output = cqlsh_cmd(scylla)
        .args(["-C", "-e", &format!("SELECT * FROM {ks}.co")])
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // -C should force ANSI codes even in pipe/non-TTY mode
    assert!(
        stdout.contains("\x1b["),
        "Expected ANSI escape codes with -C flag: {stdout}"
    );
    assert!(stdout.contains("Bob"), "Expected data in output: {stdout}");

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 3.3 — Numeric output tests (int, bigint, float, double display)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_numeric_output() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "numeric_out");

    execute_cql(
        scylla,
        &format!(
            "CREATE TABLE {ks}.nums (\
             id int PRIMARY KEY, \
             bi bigint, \
             si smallint, \
             ti tinyint, \
             f float, \
             d double)"
        ),
    )
    .success();

    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.nums (id, bi, si, ti, f, d) \
             VALUES (42, 9223372036854775807, 32000, 127, 3.14, 2.718281828)"
        ),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.nums WHERE id = 42"));

    assert!(output.contains("42"), "Expected int value: {output}");
    assert!(
        output.contains("9223372036854775807"),
        "Expected bigint value: {output}"
    );
    assert!(
        output.contains("32000"),
        "Expected smallint value: {output}"
    );
    assert!(output.contains("127"), "Expected tinyint value: {output}");
    // Float precision may vary; check prefix
    assert!(
        output.contains("3.14"),
        "Expected float value ~3.14: {output}"
    );
    assert!(
        output.contains("2.71828"),
        "Expected double value ~2.718: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 3.4 — Timestamp output test
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_timestamp_output() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "ts_out");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.ts (id int PRIMARY KEY, created timestamp)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.ts (id, created) VALUES (1, '2024-06-15 12:30:00+0000')"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.ts WHERE id = 1"));
    // Timestamp should appear in output (format may vary by timezone)
    assert!(
        output.contains("2024-06-15"),
        "Expected date portion of timestamp: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 3.5 — Boolean output test
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_boolean_output() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "bool_out");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.bools (id int PRIMARY KEY, flag boolean)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.bools (id, flag) VALUES (1, true)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.bools (id, flag) VALUES (2, false)"),
    )
    .success();

    let output_true =
        execute_cql_output(scylla, &format!("SELECT flag FROM {ks}.bools WHERE id = 1"));
    let output_false =
        execute_cql_output(scylla, &format!("SELECT flag FROM {ks}.bools WHERE id = 2"));

    assert!(
        output_true.contains("True"),
        "Expected boolean True: {output_true}"
    );
    assert!(
        output_false.contains("False"),
        "Expected boolean False: {output_false}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 3.6 — NULL output test
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_null_output() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "null_out");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.nulls (id int PRIMARY KEY, val text)"),
    )
    .success();

    // Insert with only PK → val is null
    execute_cql(scylla, &format!("INSERT INTO {ks}.nulls (id) VALUES (1)")).success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.nulls WHERE id = 1"));
    assert!(output.contains("null"), "Expected null display: {output}");

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 3.7 — String output tests (ASCII and UTF-8)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_string_output_ascii() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "str_ascii");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.strs (id int PRIMARY KEY, val text)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.strs (id, val) VALUES (1, 'Hello, World!')"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT val FROM {ks}.strs WHERE id = 1"));
    assert!(
        output.contains("Hello, World!"),
        "Expected ASCII string in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_string_output_utf8() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "str_utf8");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.strs (id int PRIMARY KEY, val text)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.strs (id, val) VALUES (1, 'こんにちは世界')"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT val FROM {ks}.strs WHERE id = 1"));
    assert!(
        output.contains("こんにちは世界"),
        "Expected UTF-8 string in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 3.8 — Blob output test
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_blob_output() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "blob_out");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.blobs (id int PRIMARY KEY, data blob)"),
    )
    .success();

    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.blobs (id, data) VALUES (1, 0xcafebabe)"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT data FROM {ks}.blobs WHERE id = 1"));
    // Blob should be displayed as hex
    assert!(
        output.to_lowercase().contains("cafebabe"),
        "Expected hex blob in output: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 3.9 — Prompt / banner test (connection banner format)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_connection_banner_format() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args(["-e", "SELECT release_version FROM system.local"])
        .output()
        .expect("failed to execute cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Banner: "Connected to <cluster> at <host>."
    assert!(
        stdout.contains("Connected to"),
        "Expected 'Connected to' in banner: {stdout}"
    );
    // Version line: "[cqlsh <version> | ...]"
    assert!(
        stdout.contains("[cqlsh"),
        "Expected '[cqlsh' version line: {stdout}"
    );
    // Help hint
    assert!(
        stdout.contains("Use HELP for help."),
        "Expected help hint: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// 3.10 — DESCRIBE output tests
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_describe_keyspace_output() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_ks");

    let output = execute_cql_output(scylla, &format!("DESCRIBE KEYSPACE {ks}"));
    assert!(
        output.contains("CREATE KEYSPACE"),
        "DESCRIBE KEYSPACE should show CREATE statement: {output}"
    );
    assert!(
        output.contains(&ks),
        "DESCRIBE KEYSPACE should include keyspace name: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_describe_table_output() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "desc_tbl");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.desc_test (id int PRIMARY KEY, name text, age int)"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("DESCRIBE TABLE {ks}.desc_test"));
    assert!(
        output.contains("CREATE TABLE"),
        "DESCRIBE TABLE should show CREATE statement: {output}"
    );
    assert!(
        output.contains("desc_test"),
        "DESCRIBE TABLE should include table name: {output}"
    );
    assert!(
        output.contains("PRIMARY KEY"),
        "DESCRIBE TABLE should show PRIMARY KEY: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_describe_cluster_output() {
    let scylla = get_scylla();

    let output = execute_cql_output(scylla, "DESCRIBE CLUSTER");
    assert!(
        output.contains("Cluster:")
            || output.contains("Partitioner:")
            || output.contains("Snitch:"),
        "DESCRIBE CLUSTER should show cluster info: {output}"
    );
}

// ---------------------------------------------------------------------------
// 3.11 — SHOW output tests
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_show_version_output() {
    let scylla = get_scylla();

    let output = execute_cql_output(scylla, "SHOW VERSION");
    assert!(
        output.contains("[cqlsh"),
        "SHOW VERSION should show cqlsh version: {output}"
    );
}

#[test]
#[ignore = "requires Docker"]
fn test_show_host_output() {
    let scylla = get_scylla();

    let output = execute_cql_output(scylla, "SHOW HOST");
    assert!(
        output.contains("Connected to"),
        "SHOW HOST should show connection info: {output}"
    );
}

// ---------------------------------------------------------------------------
// 3.12 — HELP output test (HELP is skipped in non-interactive mode)
//
// We verify that HELP doesn't cause an error in -e mode.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_help_in_noninteractive_mode() {
    let scylla = get_scylla();

    // HELP should be silently ignored in non-interactive mode
    cqlsh_cmd(scylla).args(["-e", "HELP"]).assert().success();
}

// ---------------------------------------------------------------------------
// 3.13 — Error output tests (parse errors appear on stderr)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_cql_error_output() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args([
            "--no-color",
            "-e",
            "SELECT FROM nonexistent_keyspace.nonexistent_table",
        ])
        .output()
        .expect("failed to run cqlsh-rs");

    // Command should fail
    assert!(
        !output.status.success(),
        "Invalid CQL should cause exit code 1"
    );

    // Error should appear on stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty(), "Expected error message on stderr");
}

#[test]
#[ignore = "requires Docker"]
fn test_invalid_syntax_error_output() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args(["--no-color", "-e", "SELECTT * FROM system.local"])
        .output()
        .expect("failed to run cqlsh-rs");

    assert!(
        !output.status.success(),
        "Syntax error should cause exit code 1"
    );
}

// ---------------------------------------------------------------------------
// 3.14 — Multiline test (semicolon-separated statements)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_multiline_statements() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "multi");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.ml (id int PRIMARY KEY, val text)"),
    )
    .success();

    // Multiple statements separated by semicolons
    execute_cql(
        scylla,
        &format!(
            "INSERT INTO {ks}.ml (id, val) VALUES (1, 'first'); \
             INSERT INTO {ks}.ml (id, val) VALUES (2, 'second')"
        ),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.ml"));
    assert!(output.contains("first"), "Expected first row: {output}");
    assert!(output.contains("second"), "Expected second row: {output}");

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 3.15 — Row count formatting
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_row_count_singular() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "rowcount1");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.rc (id int PRIMARY KEY, val text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.rc (id, val) VALUES (1, 'only')"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.rc"));
    assert!(
        output.contains("(1 row)") && !output.contains("(1 rows)"),
        "Expected '(1 row)' singular: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_row_count_plural() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "rowcountn");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.rc (id int PRIMARY KEY, val text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.rc (id, val) VALUES (1, 'a')"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.rc (id, val) VALUES (2, 'b')"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.rc"));
    assert!(
        output.contains("(2 rows)"),
        "Expected '(2 rows)' plural: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

#[test]
#[ignore = "requires Docker"]
fn test_row_count_zero() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "rowcount0");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.rc (id int PRIMARY KEY, val text)"),
    )
    .success();

    let output = execute_cql_output(scylla, &format!("SELECT * FROM {ks}.rc"));
    assert!(
        output.contains("(0 rows)"),
        "Expected '(0 rows)' for empty table: {output}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 3.x — Tabular format structure tests
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_tabular_pipe_separators() {
    let scylla = get_scylla();
    let ks = create_test_keyspace(scylla, "pipes");

    execute_cql(
        scylla,
        &format!("CREATE TABLE {ks}.pp (id int PRIMARY KEY, name text, city text)"),
    )
    .success();
    execute_cql(
        scylla,
        &format!("INSERT INTO {ks}.pp (id, name, city) VALUES (1, 'Alice', 'NYC')"),
    )
    .success();

    let output = cqlsh_cmd(scylla)
        .args(["--no-color", "-e", &format!("SELECT * FROM {ks}.pp")])
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Tabular output should use pipe separators between columns
    assert!(
        stdout.contains(" | "),
        "Expected pipe separators in tabular output: {stdout}"
    );
    // Header separator line should use dashes
    assert!(
        stdout.contains("---"),
        "Expected dash separator line: {stdout}"
    );

    drop_test_keyspace(scylla, &ks);
}

// ---------------------------------------------------------------------------
// 3.x — Exit code tests
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_exit_code_success() {
    let scylla = get_scylla();

    cqlsh_cmd(scylla)
        .args(["-e", "SELECT release_version FROM system.local"])
        .assert()
        .success();
}

#[test]
#[ignore = "requires Docker"]
fn test_exit_code_failure() {
    let scylla = get_scylla();

    cqlsh_cmd(scylla)
        .args(["-e", "SELECT * FROM nonexistent_ks_12345.nosuchtable"])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// 4.1 — Non-interactive mode & shell improvements
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires Docker"]
fn test_debug_command_status() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args(["--no-color", "-e", "DEBUG"])
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Debug output is currently disabled"),
        "Expected debug status in stdout: {stdout}"
    );
    assert!(output.status.success(), "Expected exit 0 for DEBUG command");
}

#[test]
#[ignore = "requires Docker"]
fn test_debug_on_command() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args(["--no-color", "-e", "DEBUG ON"])
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Now printing debug output"),
        "Expected debug-on confirmation in stdout: {stdout}"
    );
    assert!(output.status.success(), "Expected exit 0 for DEBUG ON");
}

#[test]
#[ignore = "requires Docker"]
fn test_debug_off_command() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args(["--no-color", "-e", "DEBUG OFF"])
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Disabled debug output"),
        "Expected debug-off confirmation in stdout: {stdout}"
    );
    assert!(output.status.success(), "Expected exit 0 for DEBUG OFF");
}

#[test]
#[ignore = "requires Docker"]
fn test_unicode_command() {
    let scylla = get_scylla();

    let output = cqlsh_cmd(scylla)
        .args(["--no-color", "-e", "UNICODE"])
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Encoding:"),
        "Expected 'Encoding:' in UNICODE output: {stdout}"
    );
    assert!(
        stdout.contains("utf-8"),
        "Expected 'utf-8' in UNICODE output: {stdout}"
    );
    assert!(output.status.success(), "Expected exit 0 for UNICODE command");
}

#[test]
#[ignore = "requires Docker"]
fn test_stdin_pipe_query() {
    let scylla = get_scylla();

    // Pipe a SELECT into cqlsh-rs via stdin (no -e flag)
    let output = cqlsh_cmd(scylla)
        .arg("--no-color")
        .write_stdin("SELECT key FROM system.local;\n")
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // system.local always has exactly one row with key='local'
    assert!(
        stdout.contains("local"),
        "Expected 'local' in piped SELECT output: {stdout}"
    );
    assert!(
        output.status.success(),
        "Expected exit 0 for piped SELECT: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "requires Docker"]
fn test_stdin_pipe_error_exits_1() {
    let scylla = get_scylla();

    // Invalid CQL piped via stdin should exit 1 (CQL error)
    let output = cqlsh_cmd(scylla)
        .arg("--no-color")
        .write_stdin("SELECT * FROM nonexistent_keyspace.nonexistent_table;\n")
        .output()
        .expect("failed to run cqlsh-rs");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Expected exit code 1 for piped CQL error: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "requires Docker"]
fn test_stdin_no_banner() {
    let scylla = get_scylla();

    // When stdin is piped the connection banner must NOT appear in stdout
    let output = cqlsh_cmd(scylla)
        .arg("--no-color")
        .write_stdin("SELECT key FROM system.local;\n")
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Connected to"),
        "Banner should not appear in stdout when stdin is piped: {stdout}"
    );
}

#[test]
#[ignore = "requires Docker"]
fn test_tty_flag_enables_banner_with_piped_stdin() {
    let scylla = get_scylla();

    // --tty forces interactive REPL mode even when stdin is piped.
    // Rustyline receives EOF immediately → exits cleanly after printing the banner.
    let output = cqlsh_cmd(scylla)
        .args(["--no-color", "--tty"])
        .write_stdin("")
        .output()
        .expect("failed to run cqlsh-rs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Connected to"),
        "Banner should appear with --tty even when stdin is piped: {stdout}"
    );
}

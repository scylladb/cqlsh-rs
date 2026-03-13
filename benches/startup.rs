//! Startup-path benchmarks for cqlsh-rs.
//!
//! Measures the two main flows that execute on every cqlsh-rs invocation:
//!
//! **Flow 1 — CLI argument parsing** (`CliArgs::parse_from` + `validate`)
//!   Exercises clap's derive-based parser with various argument combinations,
//!   from zero args (fastest) to a realistic "full connection" set. This is the
//!   very first thing that happens and determines how quickly the binary can
//!   begin useful work.
//!
//! **Flow 2 — Configuration loading pipeline**
//!   `CqlshrcConfig::parse` → `EnvConfig` → `MergedConfig::build`
//!   Exercises INI parsing, environment variable reading, and the four-layer
//!   merge (CLI > env > cqlshrc > defaults). Config loading is the second
//!   synchronous step before any I/O or REPL starts.
//!
//! Together these two flows represent the entire cold-start critical path.
//! Target: cold startup < 50 ms (vs Python cqlsh ~800 ms).

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use cqlsh_rs::cli::CliArgs;
use cqlsh_rs::config::{CqlshrcConfig, EnvConfig, MergedConfig};

use clap::Parser;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_cli(args: &[&str]) -> CliArgs {
    let mut full_args = vec!["cqlsh-rs"];
    full_args.extend_from_slice(args);
    CliArgs::parse_from(full_args)
}

fn default_cli() -> CliArgs {
    parse_cli(&[])
}

/// A realistic full-featured cqlshrc file content.
const SAMPLE_CQLSHRC: &str = r#"
[authentication]
username = cassandra
password = cassandra
keyspace = my_keyspace

[connection]
hostname = 10.0.0.1
port = 9042
timeout = 10
request_timeout = 10
connect_timeout = 5

[ssl]
certfile = /path/to/ca-cert.pem
validate = true
userkey = /path/to/key.pem
usercert = /path/to/usercert.pem
version = TLSv1_2

[certfiles]
172.31.10.22 = ~/keys/node0.cer.pem
172.31.8.141 = ~/keys/node1.cer.pem
172.31.5.99 = ~/keys/node2.cer.pem

[ui]
color = on
datetimeformat = %Y-%m-%d %H:%M:%S%z
timezone = UTC
float_precision = 5
double_precision = 12
max_trace_wait = 10.0
encoding = utf-8
completekey = tab
browser = firefox

[cql]
version = 3.4.7

[csv]
field_size_limit = 131072

[copy]
numprocesses = 4
maxattempts = 5
reportfrequency = 0.25

[copy-to]
pagesize = 1000
pagetimeout = 10
maxrequests = 6
maxoutputsize = -1
floatprecision = 5
doubleprecision = 12

[copy-from]
maxbatchsize = 20
minbatchsize = 10
chunksize = 5000
ingestrate = 100000
maxparseerrors = -1
maxinserterrors = 1000
preparedstatements = true
ttl = 3600

[tracing]
max_trace_wait = 10.0
"#;

/// A minimal cqlshrc with just a few settings.
const MINIMAL_CQLSHRC: &str = r#"
[connection]
hostname = 127.0.0.1
port = 9042

[authentication]
username = admin
"#;

// ---------------------------------------------------------------------------
// Flow 1: CLI argument parsing benchmarks
// ---------------------------------------------------------------------------

fn bench_cli_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_parse_args");

    // Baseline: no arguments at all
    group.bench_function("no_args", |b| {
        b.iter(|| {
            black_box(parse_cli(black_box(&[])));
        });
    });

    // Single positional host
    group.bench_function("host_only", |b| {
        b.iter(|| {
            black_box(parse_cli(black_box(&["192.168.1.1"])));
        });
    });

    // Host + port positional
    group.bench_function("host_and_port", |b| {
        b.iter(|| {
            black_box(parse_cli(black_box(&["192.168.1.1", "9043"])));
        });
    });

    // Execute mode (common non-interactive usage)
    group.bench_function("execute_mode", |b| {
        b.iter(|| {
            black_box(parse_cli(black_box(&["-e", "SELECT * FROM system.local"])));
        });
    });

    // File execution mode
    group.bench_function("file_mode", |b| {
        b.iter(|| {
            black_box(parse_cli(black_box(&["-f", "/tmp/schema.cql"])));
        });
    });

    // Realistic full connection: host, port, auth, keyspace, SSL, timeout, color
    group.bench_function("full_connection", |b| {
        b.iter(|| {
            black_box(parse_cli(black_box(&[
                "10.0.0.1",
                "9142",
                "-u",
                "admin",
                "-p",
                "secret",
                "-k",
                "production",
                "--ssl",
                "-C",
                "--connect-timeout",
                "15",
                "--request-timeout",
                "30",
                "--protocol-version",
                "4",
                "--consistency-level",
                "QUORUM",
                "--encoding",
                "utf-8",
            ])));
        });
    });

    group.finish();
}

fn bench_cli_validate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_validate");

    let valid_args = parse_cli(&[
        "10.0.0.1",
        "9142",
        "-u",
        "admin",
        "-k",
        "test",
        "--ssl",
        "--protocol-version",
        "4",
    ]);

    group.bench_function("valid_full", |b| {
        b.iter(|| {
            black_box(valid_args.validate()).unwrap();
        });
    });

    let minimal_args = default_cli();
    group.bench_function("valid_minimal", |b| {
        b.iter(|| {
            black_box(minimal_args.validate()).unwrap();
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Flow 2: Configuration loading pipeline benchmarks
// ---------------------------------------------------------------------------

fn bench_cqlshrc_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("cqlshrc_parse");

    // Empty config
    group.bench_function("empty", |b| {
        b.iter(|| {
            black_box(CqlshrcConfig::parse(black_box("")).unwrap());
        });
    });

    // Minimal config (3 keys)
    group.bench_function("minimal", |b| {
        b.iter(|| {
            black_box(CqlshrcConfig::parse(black_box(MINIMAL_CQLSHRC)).unwrap());
        });
    });

    // Full realistic config (all sections populated)
    group.bench_function("full", |b| {
        b.iter(|| {
            black_box(CqlshrcConfig::parse(black_box(SAMPLE_CQLSHRC)).unwrap());
        });
    });

    group.finish();
}

fn bench_cqlshrc_parse_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("cqlshrc_parse_scaling");

    // Measure how config parsing scales with the number of certfiles entries
    // (the only variable-length section in cqlshrc)
    for num_certfiles in [0, 10, 50, 100] {
        let mut content =
            String::from("[connection]\nhostname = 127.0.0.1\nport = 9042\n\n[certfiles]\n");
        for i in 0..num_certfiles {
            content.push_str(&format!(
                "192.168.1.{} = ~/keys/node{}.cer.pem\n",
                i % 256,
                i
            ));
        }

        group.bench_with_input(
            BenchmarkId::new("certfiles", num_certfiles),
            &content,
            |b, content| {
                b.iter(|| {
                    black_box(CqlshrcConfig::parse(black_box(content)).unwrap());
                });
            },
        );
    }

    group.finish();
}

fn bench_config_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_merge");

    let cqlshrc_path = PathBuf::from("/home/user/.cassandra/cqlshrc");

    // All defaults — fastest path, no overrides
    group.bench_function("all_defaults", |b| {
        let cli = default_cli();
        let env = EnvConfig::default();
        let cqlshrc = CqlshrcConfig::default();
        b.iter(|| {
            black_box(MergedConfig::build(
                black_box(&cli),
                black_box(&env),
                black_box(cqlshrc.clone()),
                cqlshrc_path.clone(),
            ));
        });
    });

    // CLI overrides only (no cqlshrc, no env)
    group.bench_function("cli_overrides_only", |b| {
        let cli = parse_cli(&[
            "10.0.0.1",
            "9142",
            "-u",
            "admin",
            "-p",
            "secret",
            "-k",
            "production",
            "--ssl",
            "-C",
            "--connect-timeout",
            "15",
        ]);
        let env = EnvConfig::default();
        let cqlshrc = CqlshrcConfig::default();
        b.iter(|| {
            black_box(MergedConfig::build(
                black_box(&cli),
                black_box(&env),
                black_box(cqlshrc.clone()),
                cqlshrc_path.clone(),
            ));
        });
    });

    // Full merge — all layers populated (worst case)
    group.bench_function("full_merge", |b| {
        let cli = parse_cli(&[
            "cli-host",
            "9999",
            "-u",
            "cli-user",
            "--connect-timeout",
            "99",
            "--encoding",
            "utf-16",
        ]);
        let env = EnvConfig {
            host: Some("env-host".to_string()),
            port: Some(8888),
            connect_timeout: Some(88),
            request_timeout: Some(88),
            ssl_certfile: Some("/path/to/cert.pem".to_string()),
            ssl_validate: Some(true),
            history_file: Some("/home/user/.cql_history".to_string()),
        };
        let cqlshrc = CqlshrcConfig::parse(SAMPLE_CQLSHRC).unwrap();
        b.iter(|| {
            black_box(MergedConfig::build(
                black_box(&cli),
                black_box(&env),
                black_box(cqlshrc.clone()),
                cqlshrc_path.clone(),
            ));
        });
    });

    group.finish();
}

fn bench_cqlshrc_load_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("cqlshrc_load_file");

    // Benchmark loading from an actual file (includes filesystem I/O)
    let dir = tempfile::tempdir().unwrap();

    // Nonexistent file (fast path — returns default)
    let nonexistent = dir.path().join("nonexistent");
    group.bench_function("nonexistent_file", |b| {
        b.iter(|| {
            black_box(CqlshrcConfig::load(black_box(&nonexistent)).unwrap());
        });
    });

    // Minimal file on disk
    let minimal_path = dir.path().join("minimal_cqlshrc");
    std::fs::write(&minimal_path, MINIMAL_CQLSHRC).unwrap();
    group.bench_function("minimal_file", |b| {
        b.iter(|| {
            black_box(CqlshrcConfig::load(black_box(&minimal_path)).unwrap());
        });
    });

    // Full file on disk
    let full_path = dir.path().join("full_cqlshrc");
    std::fs::write(&full_path, SAMPLE_CQLSHRC).unwrap();
    group.bench_function("full_file", |b| {
        b.iter(|| {
            black_box(CqlshrcConfig::load(black_box(&full_path)).unwrap());
        });
    });

    group.finish();
}

fn bench_end_to_end_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_startup");

    let dir = tempfile::tempdir().unwrap();

    // Simulate the full startup path: parse CLI → load cqlshrc → merge config
    // This is what main() does before any I/O or REPL.

    // Minimal startup: no cqlshrc file, default args
    let nonexistent = dir.path().join("nonexistent");
    group.bench_function("minimal", |b| {
        b.iter(|| {
            let cli = parse_cli(&["--cqlshrc", nonexistent.to_str().unwrap()]);
            let _ = cli.validate();
            let cqlshrc = CqlshrcConfig::load(&nonexistent).unwrap();
            let env = EnvConfig::default();
            black_box(MergedConfig::build(
                &cli,
                &env,
                cqlshrc,
                nonexistent.clone(),
            ));
        });
    });

    // Full startup: cqlshrc file on disk, many CLI args
    let full_path = dir.path().join("full_cqlshrc");
    std::fs::write(&full_path, SAMPLE_CQLSHRC).unwrap();
    group.bench_function("full", |b| {
        b.iter(|| {
            let cli = parse_cli(&[
                "10.0.0.1",
                "9142",
                "-u",
                "admin",
                "-p",
                "secret",
                "-k",
                "production",
                "--ssl",
                "-C",
                "--connect-timeout",
                "15",
                "--request-timeout",
                "30",
                "--protocol-version",
                "4",
                "--cqlshrc",
                full_path.to_str().unwrap(),
            ]);
            let _ = cli.validate();
            let cqlshrc = CqlshrcConfig::load(&full_path).unwrap();
            let env = EnvConfig {
                host: Some("env-host".to_string()),
                port: Some(8888),
                connect_timeout: Some(88),
                ..EnvConfig::default()
            };
            black_box(MergedConfig::build(&cli, &env, cqlshrc, full_path.clone()));
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion harness
// ---------------------------------------------------------------------------

criterion_group!(cli_benches, bench_cli_parsing, bench_cli_validate,);

criterion_group!(
    config_benches,
    bench_cqlshrc_parse,
    bench_cqlshrc_parse_scaling,
    bench_config_merge,
    bench_cqlshrc_load_file,
    bench_end_to_end_startup,
);

criterion_main!(cli_benches, config_benches);

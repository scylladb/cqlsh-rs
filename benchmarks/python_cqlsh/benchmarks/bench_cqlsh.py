"""Benchmarks for Python cqlsh (from PyPI) against a live ScyllaDB instance.

Measures the end-to-end performance of the Python cqlsh CLI for common
operations. These benchmarks serve as the baseline for comparing against
the Rust cqlsh-rs implementation.

Benchmark groups:
  - version:       `cqlsh --version` (cold startup cost)
  - connect-query: Connect + execute a simple query
  - select-single: SELECT a single row by primary key
  - select-multi:  SELECT multiple rows
  - insert:        INSERT a single row
  - describe:      DESCRIBE KEYSPACES
"""

from __future__ import annotations

import subprocess
from typing import Any

import pytest


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _run_cqlsh(
    cqlsh_bin: str,
    host: str,
    port: int,
    statement: str,
    *,
    timeout: float = 30.0,
) -> subprocess.CompletedProcess[str]:
    """Execute a CQL statement via the cqlsh CLI."""
    return subprocess.run(
        [cqlsh_bin, host, str(port), "-e", statement],
        capture_output=True,
        text=True,
        timeout=timeout,
    )


# ---------------------------------------------------------------------------
# version — measures pure startup overhead
# ---------------------------------------------------------------------------


@pytest.mark.benchmark(group="version")
def test_cqlsh_version(benchmark: Any, cqlsh_bin: str) -> None:
    """Benchmark: cqlsh --version (startup cost only, no DB connection)."""

    def _version() -> None:
        result = subprocess.run(
            [cqlsh_bin, "--version"],
            capture_output=True,
            text=True,
            timeout=30.0,
        )
        assert result.returncode == 0

    benchmark(_version)


# ---------------------------------------------------------------------------
# connect-query — connect and run a trivial query
# ---------------------------------------------------------------------------


@pytest.mark.benchmark(group="connect-query")
def test_cqlsh_connect_and_query(
    benchmark: Any,
    cqlsh_bin: str,
    scylla_host_port: tuple[str, int],
    bench_keyspace: str,
) -> None:
    """Benchmark: connect to ScyllaDB and SELECT now() FROM system.local."""
    host, port = scylla_host_port

    def _query() -> None:
        result = _run_cqlsh(
            cqlsh_bin, host, port, "SELECT now() FROM system.local;"
        )
        assert result.returncode == 0

    benchmark(_query)


# ---------------------------------------------------------------------------
# select-single — SELECT one row by primary key
# ---------------------------------------------------------------------------


@pytest.mark.benchmark(group="select-single")
def test_cqlsh_select_single_row(
    benchmark: Any,
    cqlsh_bin: str,
    scylla_host_port: tuple[str, int],
    bench_keyspace: str,
) -> None:
    """Benchmark: SELECT a single row by primary key."""
    host, port = scylla_host_port

    def _select() -> None:
        result = _run_cqlsh(
            cqlsh_bin,
            host,
            port,
            f"SELECT * FROM {bench_keyspace}.bench_kv WHERE key = 'key_0';",
        )
        assert result.returncode == 0

    benchmark(_select)


# ---------------------------------------------------------------------------
# select-multi — SELECT multiple rows
# ---------------------------------------------------------------------------


@pytest.mark.benchmark(group="select-multi")
def test_cqlsh_select_multi_rows(
    benchmark: Any,
    cqlsh_bin: str,
    scylla_host_port: tuple[str, int],
    bench_keyspace: str,
) -> None:
    """Benchmark: SELECT all rows from bench_kv (100 rows)."""
    host, port = scylla_host_port

    def _select() -> None:
        result = _run_cqlsh(
            cqlsh_bin,
            host,
            port,
            f"SELECT * FROM {bench_keyspace}.bench_kv;",
        )
        assert result.returncode == 0

    benchmark(_select)


# ---------------------------------------------------------------------------
# insert — INSERT a single row
# ---------------------------------------------------------------------------


@pytest.mark.benchmark(group="insert")
def test_cqlsh_insert_row(
    benchmark: Any,
    cqlsh_bin: str,
    scylla_host_port: tuple[str, int],
    bench_keyspace: str,
) -> None:
    """Benchmark: INSERT a single row."""
    host, port = scylla_host_port
    counter = {"n": 0}

    def _insert() -> None:
        n = counter["n"]
        counter["n"] += 1
        result = _run_cqlsh(
            cqlsh_bin,
            host,
            port,
            (
                f"INSERT INTO {bench_keyspace}.bench_kv (key, value) "
                f"VALUES ('bench_insert_{n}', 'val_{n}');"
            ),
        )
        assert result.returncode == 0

    benchmark(_insert)


# ---------------------------------------------------------------------------
# describe — DESCRIBE KEYSPACES
# ---------------------------------------------------------------------------


@pytest.mark.benchmark(group="describe")
def test_cqlsh_describe_keyspaces(
    benchmark: Any,
    cqlsh_bin: str,
    scylla_host_port: tuple[str, int],
    bench_keyspace: str,
) -> None:
    """Benchmark: DESCRIBE KEYSPACES."""
    host, port = scylla_host_port

    def _describe() -> None:
        result = _run_cqlsh(
            cqlsh_bin, host, port, "DESCRIBE KEYSPACES;"
        )
        assert result.returncode == 0

    benchmark(_describe)


# ---------------------------------------------------------------------------
# select-system-schema — heavier metadata query
# ---------------------------------------------------------------------------


@pytest.mark.benchmark(group="select-system-schema")
def test_cqlsh_select_system_schema(
    benchmark: Any,
    cqlsh_bin: str,
    scylla_host_port: tuple[str, int],
    bench_keyspace: str,
) -> None:
    """Benchmark: SELECT from system_schema.tables (metadata-heavy)."""
    host, port = scylla_host_port

    def _select() -> None:
        result = _run_cqlsh(
            cqlsh_bin,
            host,
            port,
            "SELECT keyspace_name, table_name FROM system_schema.tables;",
        )
        assert result.returncode == 0

    benchmark(_select)

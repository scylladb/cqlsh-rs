"""Shared fixtures for Python cqlsh benchmarks.

Provides a ScyllaDB testcontainer and helper functions to run cqlsh commands
against the live database. Following the pattern from fruch/coodie.
"""

from __future__ import annotations

import shutil
import subprocess
import time
from typing import Any

import pytest


class LocalhostTranslator:
    """Translate any discovered host address to 127.0.0.1.

    When ScyllaDB runs in Docker, system.local advertises the container-internal
    IP. By translating every address back to 127.0.0.1 the driver always reaches
    the node through the mapped host port.
    """

    def translate(self, addr: str) -> str:
        return "127.0.0.1"


@pytest.fixture(scope="session")
def scylla_container():
    """Start a ScyllaDB container once for the entire benchmark session."""
    try:
        from testcontainers.core.container import DockerContainer
        from testcontainers.core.waiting_utils import wait_for_logs
    except ImportError as exc:
        pytest.skip(f"testcontainers not installed: {exc}")

    with (
        DockerContainer("scylladb/scylla:6.2")
        .with_command(
            "--smp 1 --memory 512M --developer-mode 1 "
            "--skip-wait-for-gossip-to-settle=0"
        )
        .with_exposed_ports(9042) as container
    ):
        wait_for_logs(
            container, "Starting listening for CQL clients", timeout=120
        )
        # Give ScyllaDB a moment to fully initialize after log message
        time.sleep(2)
        yield container


@pytest.fixture(scope="session")
def scylla_host_port(scylla_container: Any) -> tuple[str, int]:
    """Return (host, port) for connecting to the ScyllaDB container."""
    port = int(scylla_container.get_exposed_port(9042))
    return "127.0.0.1", port


@pytest.fixture(scope="session")
def cqlsh_bin() -> str:
    """Return the path to the cqlsh binary installed from PyPI."""
    path = shutil.which("cqlsh")
    if path is None:
        pytest.skip("cqlsh not found in PATH — install with: pip install cqlsh")
    return path


@pytest.fixture(scope="session")
def bench_keyspace(scylla_host_port: tuple[str, int], cqlsh_bin: str) -> str:
    """Create the benchmark keyspace and a sample table, return keyspace name."""
    host, port = scylla_host_port
    keyspace = "bench_ks"

    # Create keyspace
    _run_cqlsh(
        cqlsh_bin,
        host,
        port,
        (
            f"CREATE KEYSPACE IF NOT EXISTS {keyspace} "
            "WITH replication = {'class': 'SimpleStrategy', 'replication_factor': '1'};"
        ),
    )

    # Create a simple table for query benchmarks
    _run_cqlsh(
        cqlsh_bin,
        host,
        port,
        (
            f"CREATE TABLE IF NOT EXISTS {keyspace}.bench_kv ("
            "  key text PRIMARY KEY,"
            "  value text"
            ");"
        ),
    )

    # Seed some rows for read benchmarks
    for i in range(100):
        _run_cqlsh(
            cqlsh_bin,
            host,
            port,
            (
                f"INSERT INTO {keyspace}.bench_kv (key, value) "
                f"VALUES ('key_{i}', 'value_{i}');"
            ),
        )

    return keyspace


def _run_cqlsh(
    cqlsh_bin: str,
    host: str,
    port: int,
    statement: str,
    *,
    timeout: float = 30.0,
) -> subprocess.CompletedProcess[str]:
    """Execute a CQL statement via the cqlsh CLI."""
    result = subprocess.run(
        [cqlsh_bin, host, str(port), "-e", statement],
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"cqlsh failed (rc={result.returncode}):\n"
            f"stdout: {result.stdout}\n"
            f"stderr: {result.stderr}"
        )
    return result

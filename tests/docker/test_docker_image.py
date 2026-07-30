"""End-to-end tests for the cqlsh-rs Docker image.

These tests treat the image as a black box: they only run `docker run` and
assert on what a user would see. They exist because the Rust test-suite runs
against a locally built binary on a glibc host and therefore cannot catch
image-level regressions — the class of bug reported in
https://github.com/scylladb/cqlsh-rs/issues/175, where the Alpine image only
had BusyBox `less` and every interactive `SELECT` rendered nothing.

Two kinds of checks:

* **batch**  — `docker run --rm IMAGE -e "SELECT ..."`, plain subprocess.
* **interactive** — the image driven through a real PTY with `pexpect`, so the
  REPL takes the tty path: banner, prompt, pager, tab completion, `exit`.

Python rather than Rust because none of this touches cqlsh-rs internals — the
image is a black box driven over a pty, and `pexpect` is a mature driver for
exactly that. See CONTRIBUTING.md ("Testing the image") for the full rationale.

Configuration (env vars, all optional):

    CQLSH_DOCKER_IMAGE     image under test        (default: cqlsh-rs:ci)
    CQLSH_DOCKER_NETWORK   docker network to join  (default: cqlsh-ci-net)
    CQLSH_DB_HOST          DB host as seen from inside that network
                                                   (default: cqlsh-ci-db)
    CQLSH_DB_PORT          CQL port                (default: 9042)

Run locally (from this directory; uv resolves the deps from pyproject.toml):

    docker network create cqlsh-ci-net
    docker run -d --name cqlsh-ci-db --network cqlsh-ci-net \\
        scylladb/scylla:2025.1 --smp 1 --memory 512M --overprovisioned 1
    uv run pytest
"""

from __future__ import annotations

import os
import re
import subprocess

import pexpect
import pytest

IMAGE = os.environ.get("CQLSH_DOCKER_IMAGE", "cqlsh-rs:ci")
NETWORK = os.environ.get("CQLSH_DOCKER_NETWORK", "cqlsh-ci-net")
DB_HOST = os.environ.get("CQLSH_DB_HOST", "cqlsh-ci-db")
DB_PORT = os.environ.get("CQLSH_DB_PORT", "9042")

# Anything BusyBox `less` prints when it chokes on a GNU-only flag. If one of
# these shows up in interactive output, the pager is broken again (#175).
PAGER_BREAKAGE = [
    "unrecognized option",
    "invalid option",
    "BusyBox",
    "Usage: less",
]

PROMPT = re.compile(r"cqlsh(:\w+)?> ")

# Markers that can only come from rendered query output, never from the echoed
# input line. The REPL redraws what you type (with syntax highlighting), so
# expecting a column name would match the echo and pass even when the pager
# swallowed every row.
ROW_COUNT = re.compile(r"\(\d+ rows?\)")
TABLE_BORDER = re.compile(r"[+|-]{5,}")

# A stand-in pager for the "batch must not page" test. cqlsh-rs honours $PAGER
# (src/pager.rs) and splits it on whitespace, so it has to be a single word
# with no arguments. `wc` is in BusyBox, exits 0 (so cqlsh-rs does not fall
# through to `less`), and consumes the rows instead of printing them — if
# batch mode ever invoked a pager, the table would be replaced by byte counts.
SENTINEL_PAGER = "wc"


def docker_run(*args: str, timeout: int = 60) -> subprocess.CompletedProcess:
    """Run the image non-interactively and return the completed process."""
    return subprocess.run(
        ["docker", "run", "--rm", "--network", NETWORK, IMAGE, *args],
        capture_output=True,
        text=True,
        timeout=timeout,
    )


class _Transcript:
    """Collects PTY output for assertions and for failure diagnostics."""

    def __init__(self) -> None:
        self.buf: list[str] = []

    def write(self, data: str) -> None:
        self.buf.append(data)

    def flush(self) -> None:
        pass

    def text(self) -> str:
        return "".join(self.buf)

    def reset(self) -> None:
        self.buf.clear()


LOG = _Transcript()


@pytest.fixture(autouse=True)
def _fresh_transcript():
    LOG.reset()
    yield
    # pytest only shows this for failing tests, where it is exactly what you
    # want to read: everything the terminal received.
    print(LOG.text())


@pytest.fixture
def spawn_interactive():
    """Factory for the image attached to a PTY, connected to the test database.

    `docker run -it` needs a real tty on our side; pexpect provides one, which
    is what makes the shell take its interactive code path (banner, prompt,
    colors, pager) instead of the batch path.

    Children are force-closed on teardown so a failed assertion mid-session
    cannot leave a container holding a pty in CI.
    """
    children: list[pexpect.spawn] = []

    def _spawn(*extra_args: str) -> pexpect.spawn:
        argv = [
            "run", "--rm", "-it",
            "--network", NETWORK,
            IMAGE,
            *extra_args,
            DB_HOST, DB_PORT,
        ]  # fmt: skip
        child = pexpect.spawn(
            "docker", argv, timeout=60, encoding="utf-8", dimensions=(24, 120)
        )
        # Keep a transcript so failures show what the user would have seen.
        child.logfile_read = LOG
        children.append(child)
        return child

    yield _spawn

    for child in children:
        if child.isalive():
            child.close(force=True)


@pytest.fixture
def spawn_batch():
    """Factory for a *batch* invocation that still gets a tty (`docker run -t`).

    The point is the combination: `-e` should take the non-interactive path
    regardless of what stdout is attached to, so this is the case that
    distinguishes "batch never pages" from "batch happens not to page because
    nothing was a terminal".
    """
    children: list[pexpect.spawn] = []

    def _spawn(*args: str) -> pexpect.spawn:
        argv = [
            "run", "--rm", "-t",
            "--network", NETWORK,
            "--env", f"PAGER={SENTINEL_PAGER}",
            IMAGE,
            *args,
            DB_HOST, DB_PORT,
        ]  # fmt: skip
        child = pexpect.spawn(
            "docker", argv, timeout=60, encoding="utf-8", dimensions=(24, 120)
        )
        child.logfile_read = LOG
        children.append(child)
        return child

    yield _spawn

    for child in children:
        if child.isalive():
            child.close(force=True)


def assert_pager_healthy(transcript: str) -> None:
    for marker in PAGER_BREAKAGE:
        assert marker not in transcript, (
            f"pager error {marker!r} in output — the image's `less` does not "
            f"accept the flags cqlsh-rs passes it (see #175)\n{transcript}"
        )


def expect_rows(child: pexpect.spawn, marker=ROW_COUNT) -> None:
    """Wait for rendered query output, failing fast on a broken pager.

    Listing the pager's own error messages as alternatives means a broken pager
    reports itself immediately instead of burning the whole timeout and
    surfacing as a bare `pexpect.TIMEOUT` with no explanation.
    """
    index = child.expect([marker, *PAGER_BREAKAGE, pexpect.TIMEOUT], timeout=45)
    assert index == 0, (
        f"expected query output matching {getattr(marker, 'pattern', marker)!r} "
        f"but the pager failed to display it (see #175)"
        f"\n--- transcript ---\n{LOG.text()}"
    )


# --------------------------------------------------------------------------
# Image contents / batch mode
# --------------------------------------------------------------------------


def test_version_flag():
    proc = docker_run("--version")
    assert proc.returncode == 0, proc.stderr
    assert "cqlsh" in proc.stdout.lower()


def test_image_ships_real_less():
    """The pager must not be the BusyBox applet.

    cqlsh-rs degrades gracefully when it detects BusyBox, but the image is
    expected to carry real `less` so users get colors and the prompt line.
    """
    proc = subprocess.run(
        ["docker", "run", "--rm", "--entrypoint", "less", IMAGE, "--version"],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert proc.returncode == 0, f"`less --version` failed: {proc.stderr}"
    assert "BusyBox" not in proc.stdout + proc.stderr
    assert proc.stdout.startswith("less ")


def test_batch_query():
    proc = docker_run(
        "-e", "SELECT release_version FROM system.local", DB_HOST, DB_PORT
    )
    assert proc.returncode == 0, proc.stderr
    assert "release_version" in proc.stdout


def test_batch_query_is_not_paged_without_a_tty():
    """Batch mode with no terminal attached: plain output, no pager."""
    proc = docker_run("-e", "SELECT * FROM system.local", DB_HOST, DB_PORT)
    assert proc.returncode == 0, proc.stderr
    assert_pager_healthy(proc.stdout + proc.stderr)


def test_batch_query_is_not_paged_on_a_tty(spawn_batch):
    """Batch mode must not page even when stdout *is* a terminal.

    The no-tty case above is the easy half — nothing pages when there is no
    terminal to page to. This is the half that can actually regress: if
    cqlsh-rs ever decided to page `-e` output whenever stdout happens to be a
    tty, `$PAGER` would swallow the table and print byte counts instead, so
    asserting the rendered rows are *still there* catches it.

    Waiting for EOF is part of the assertion: a pager on a tty would sit
    waiting for `q` and the run would time out instead of finishing.
    """
    child = spawn_batch("-e", "SELECT * FROM system.local")
    child.expect(pexpect.EOF)
    child.close()

    out = LOG.text()
    assert TABLE_BORDER.search(out), (
        f"no table in batch output on a tty — $PAGER={SENTINEL_PAGER} appears to "
        f"have been invoked, i.e. batch mode paged\n--- transcript ---\n{out}"
    )
    assert ROW_COUNT.search(out), (
        f"no '(N rows)' footer in batch output on a tty — $PAGER="
        f"{SENTINEL_PAGER} swallowed it\n--- transcript ---\n{out}"
    )
    assert_pager_healthy(out)
    assert child.exitstatus == 0, f"exit status {child.exitstatus}"


def test_runs_as_non_root():
    proc = subprocess.run(
        ["docker", "run", "--rm", "--entrypoint", "id", IMAGE, "-u"],
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert proc.stdout.strip() == "1000"


# --------------------------------------------------------------------------
# Interactive mode (PTY)
# --------------------------------------------------------------------------


def test_interactive_banner_and_prompt(spawn_interactive):
    child = spawn_interactive()
    child.expect(PROMPT)
    assert "Connected to" in LOG.text()
    child.sendline("exit")
    child.expect(pexpect.EOF)


def test_interactive_select_displays_rows_through_pager(spawn_interactive):
    """The regression test for #175: rows must actually reach the screen.

    Colors are forced on with `-C` because the color path is what made the
    `-R` flag semantics matter.
    """
    child = spawn_interactive("-C")
    child.expect(PROMPT)
    child.sendline("SELECT release_version FROM system.local;")
    expect_rows(child, TABLE_BORDER)  # the table frame drawn inside the pager
    expect_rows(child)  # ... and the "(1 row)" footer after it
    assert_pager_healthy(LOG.text())
    # 'q' quits the pager and returns to the prompt.
    child.send("q")
    child.expect(PROMPT)
    child.sendline("exit")
    child.expect(pexpect.EOF)


def test_interactive_paging_off_prints_inline(spawn_interactive):
    child = spawn_interactive()
    child.expect(PROMPT)
    child.sendline("PAGING OFF")
    child.expect(PROMPT)
    child.sendline("SELECT release_version FROM system.local;")
    expect_rows(child)
    child.expect(PROMPT)  # no pager to quit — prompt comes straight back
    assert_pager_healthy(LOG.text())
    child.sendline("exit")
    child.expect(pexpect.EOF)


def test_interactive_describe_keyspaces(spawn_interactive):
    child = spawn_interactive()
    child.expect(PROMPT)
    child.sendline("PAGING OFF")
    child.expect(PROMPT)
    child.sendline("DESCRIBE KEYSPACES;")
    child.expect("system")
    child.sendline("exit")
    child.expect(pexpect.EOF)


def test_interactive_tab_completion(spawn_interactive):
    """Line editing must work — it needs a real tty and terminal size."""
    child = spawn_interactive()
    child.expect(PROMPT)
    child.send("SELE\t")
    child.expect("SELECT")
    child.sendcontrol("c")
    child.sendline("exit")
    child.expect(pexpect.EOF)


def test_interactive_ctrl_d_exits_cleanly(spawn_interactive):
    child = spawn_interactive()
    child.expect(PROMPT)
    child.sendcontrol("d")
    child.expect(pexpect.EOF)
    child.close()
    assert child.exitstatus == 0, f"exit status {child.exitstatus}"

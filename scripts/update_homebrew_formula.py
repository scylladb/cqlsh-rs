#!/usr/bin/env python3
"""Update Formula/cqlsh-rs.rb to point at a released version.

Reads the SHA256SUMS.txt asset published with the GitHub Release and rewrites
the ``version``, ``url`` and ``sha256`` fields of the Homebrew formula in place.

Usage:
    python3 scripts/update_homebrew_formula.py 0.5.14
    python3 scripts/update_homebrew_formula.py 0.5.14 --check
    python3 scripts/update_homebrew_formula.py 0.5.14 --sums /path/to/SHA256SUMS.txt

Exit codes:
    0  formula updated (or already up to date)
    1  error (missing assets, unparsable formula, ...)
    2  ``--check`` was given and the formula is out of date
"""

from __future__ import annotations

import argparse
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_FORMULA = REPO_ROOT / "Formula" / "cqlsh-rs.rb"
DEFAULT_REPO = "scylladb/cqlsh-rs"

# Targets that the Homebrew formula covers (Windows is not a Homebrew platform).
TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-musl",
    "x86_64-unknown-linux-musl",
)


def fetch_checksums(repo: str, version: str, sums_file: Path | None) -> dict[str, str]:
    """Return a ``{filename: sha256}`` mapping for the release's assets."""
    if sums_file is not None:
        text = sums_file.read_text(encoding="utf-8")
    else:
        url = f"https://github.com/{repo}/releases/download/v{version}/SHA256SUMS.txt"
        try:
            with urllib.request.urlopen(url, timeout=60) as response:  # noqa: S310
                text = response.read().decode("utf-8")
        except urllib.error.HTTPError as exc:
            sys.exit(f"error: cannot download {url}: HTTP {exc.code}")
        except urllib.error.URLError as exc:
            sys.exit(f"error: cannot download {url}: {exc.reason}")

    checksums: dict[str, str] = {}
    for line in text.splitlines():
        parts = line.split()
        if len(parts) == 2:
            digest, filename = parts
            checksums[filename.lstrip("*")] = digest
    if not checksums:
        sys.exit("error: no checksums parsed from SHA256SUMS.txt")
    return checksums


def render(formula: str, repo: str, version: str, checksums: dict[str, str]) -> str:
    """Return ``formula`` with version/url/sha256 rewritten for ``version``."""
    updated, count = re.subn(
        r'^(\s*)version "[^"]*"',
        rf'\g<1>version "{version}"',
        formula,
        count=1,
        flags=re.MULTILINE,
    )
    if count != 1:
        sys.exit("error: no `version \"...\"` line found in formula")

    for target in TARGETS:
        archive = f"cqlsh-rs-{version}-{target}.tar.gz"
        digest = checksums.get(archive)
        if digest is None:
            sys.exit(f"error: {archive} missing from SHA256SUMS.txt")

        url = f"https://github.com/{repo}/releases/download/v{version}/{archive}"
        pattern = (
            rf'url "[^"]*-{re.escape(target)}\.tar\.gz"\n(?P<indent>[ \t]*)sha256 "[^"]*"'
        )
        replacement = f'url "{url}"\n\\g<indent>sha256 "{digest}"'
        updated, count = re.subn(pattern, replacement, updated, count=1)
        if count != 1:
            sys.exit(f"error: no url/sha256 pair found for target {target}")

    return updated


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", help="released version, without the leading 'v'")
    parser.add_argument(
        "--formula", type=Path, default=DEFAULT_FORMULA, help="path to the formula"
    )
    parser.add_argument("--repo", default=DEFAULT_REPO, help="owner/name of the GitHub repo")
    parser.add_argument(
        "--sums", type=Path, help="use a local SHA256SUMS.txt instead of downloading it"
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="exit 2 if the formula would change, without writing it",
    )
    args = parser.parse_args()

    version = args.version.lstrip("v")
    formula = args.formula.read_text(encoding="utf-8")
    checksums = fetch_checksums(args.repo, version, args.sums)
    updated = render(formula, args.repo, version, checksums)

    if updated == formula:
        print(f"Formula already up to date for v{version}")
        return 0

    if args.check:
        print(f"Formula is out of date: does not match v{version}", file=sys.stderr)
        return 2

    args.formula.write_text(updated, encoding="utf-8")
    print(f"Updated {args.formula.relative_to(REPO_ROOT)} to v{version}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

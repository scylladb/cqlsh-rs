#!/usr/bin/env python3
"""Parse bencher output into a grouped Markdown summary for GitHub Actions Job Summary.

Usage:
    cargo bench --bench startup -- --output-format bencher | python3 scripts/format_bench_summary.py
"""

import re
import sys
from collections import OrderedDict

# Bencher line: test <name> ... bench: <ns> ns/iter (+/- <variance>)
BENCH_RE = re.compile(
    r"^test\s+(\S+)\s+\.\.\.\s+bench:\s+([\d,]+)\s+ns/iter\s+\(\+/-\s+([\d,]+)\)$"
)

# Key benchmarks to highlight at the top (name substring -> display label)
KEY_BENCHMARKS = OrderedDict([
    ("end_to_end_startup/full", "End-to-end startup (full)"),
    ("end_to_end_startup/minimal", "End-to-end startup (minimal)"),
    ("cli_parse_args/full_connection", "CLI parse (full connection)"),
    ("cqlshrc_parse/full", "Config parse (full cqlshrc)"),
])


def humanize_ns(ns: int) -> str:
    """Convert nanoseconds to human-friendly units."""
    if ns >= 1_000_000:
        return f"{ns / 1_000_000:.1f} ms"
    if ns >= 1_000:
        return f"{ns / 1_000:.1f} \u00b5s"
    return f"{ns} ns"


def parse_bencher(stream):
    """Parse bencher lines into (name, ns, variance_ns) tuples."""
    results = []
    for line in stream:
        m = BENCH_RE.match(line.strip())
        if m:
            name = m.group(1)
            ns = int(m.group(2).replace(",", ""))
            var = int(m.group(3).replace(",", ""))
            results.append((name, ns, var))
    return results


def group_results(results):
    """Group results by prefix before '/'."""
    groups = OrderedDict()
    for name, ns, var in results:
        if "/" in name:
            group, bench = name.split("/", 1)
        else:
            group, bench = name, name
        groups.setdefault(group, []).append((bench, ns, var))
    return groups


def format_markdown(results):
    """Produce grouped Markdown tables."""
    lines = ["## Benchmark Summary", ""]

    # Key results
    key_entries = []
    result_map = {name: (ns, var) for name, ns, var in results}
    for pattern, label in KEY_BENCHMARKS.items():
        for name, ns, var in results:
            if pattern in name:
                key_entries.append((label, ns))
                break

    if key_entries:
        lines.append("### Key Results")
        lines.append("")
        lines.append("| Metric | Time |")
        lines.append("|--------|------|")
        for label, ns in key_entries:
            lines.append(f"| {label} | {humanize_ns(ns)} |")
        lines.append("")

    # Grouped tables
    groups = group_results(results)
    for group, entries in groups.items():
        title = group.replace("_", " ").title()
        lines.append(f"### {title}")
        lines.append("")
        lines.append("| Benchmark | Time | \u00b1 |")
        lines.append("|-----------|------|---|")
        for bench, ns, var in entries:
            lines.append(f"| {bench} | {humanize_ns(ns)} | {humanize_ns(var)} |")
        lines.append("")

    # Dashboard link
    lines.append("> Historical trends: https://fruch.github.io/cqlsh-rs/dev/bench/")
    lines.append("")

    return "\n".join(lines)


def main():
    results = parse_bencher(sys.stdin)
    if not results:
        print("No benchmark results found.", file=sys.stderr)
        sys.exit(1)
    print(format_markdown(results))


if __name__ == "__main__":
    main()

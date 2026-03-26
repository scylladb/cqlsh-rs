#!/usr/bin/env python3
"""Parse Criterion benchmark output and produce JSON + Markdown summary.

Supports Criterion's default format:
  <group>/<bench>    time:   [<low> <unit> <mid> <unit> <high> <unit>]

Also supports legacy bencher format:
  test <name> ... bench: <ns> ns/iter (+/- <variance>)

Usage:
    cargo bench 2>&1 | python3 scripts/format_bench_summary.py
    # Writes output.json (for benchmark-action) and prints Markdown to stdout
"""

import json
import re
import sys
from collections import OrderedDict

# Criterion line: <name>     time:   [1.234 µs 1.256 µs 1.278 µs]
CRITERION_RE = re.compile(
    r"^(\S+)\s+time:\s+\[[\d.]+ \S+ ([\d.]+) (\S+) [\d.]+ \S+\]$"
)

# Bencher line: test <name> ... bench: <ns> ns/iter (+/- <variance>)
BENCH_RE = re.compile(
    r"^test\s+(\S+)\s+\.\.\.\s+bench:\s+([\d,]+)\s+ns/iter\s+\(\+/-\s+([\d,]+)\)$"
)

UNIT_TO_NS = {"ns": 1, "µs": 1_000, "us": 1_000, "ms": 1_000_000, "s": 1_000_000_000}


def humanize_ns(ns: int) -> str:
    if ns >= 1_000_000:
        return f"{ns / 1_000_000:.1f} ms"
    if ns >= 1_000:
        return f"{ns / 1_000:.1f} µs"
    return f"{ns} ns"


def parse_results(stream):
    """Parse benchmark lines into (name, ns) tuples."""
    results = []
    for line in stream:
        stripped = line.strip()
        m = CRITERION_RE.match(stripped)
        if m:
            name = m.group(1)
            value = float(m.group(2))
            unit = m.group(3)
            ns = int(value * UNIT_TO_NS.get(unit, 1))
            results.append((name, ns))
            continue
        m = BENCH_RE.match(stripped)
        if m:
            name = m.group(1)
            ns = int(m.group(2).replace(",", ""))
            results.append((name, ns))
    return results


def to_benchmark_action_json(results):
    """Produce JSON for benchmark-action/github-action-benchmark (customSmallerIsBetter)."""
    return [{"name": name, "unit": "ns", "value": ns} for name, ns in results]


def to_markdown(results):
    """Produce grouped Markdown tables."""
    lines = ["## Benchmark Summary", ""]
    groups = OrderedDict()
    for name, ns in results:
        if "/" in name:
            group, bench = name.split("/", 1)
        else:
            group, bench = name, name
        groups.setdefault(group, []).append((bench, ns))

    for group, entries in groups.items():
        title = group.replace("_", " ").title()
        lines.append(f"### {title}")
        lines.append("")
        lines.append("| Benchmark | Time |")
        lines.append("|-----------|------|")
        for bench, ns in entries:
            lines.append(f"| {bench} | {humanize_ns(ns)} |")
        lines.append("")

    lines.append("> Historical trends: https://fruch.github.io/cqlsh-rs/dev/bench/")
    return "\n".join(lines)


def main():
    results = parse_results(sys.stdin)
    if not results:
        # Write empty JSON and exit
        with open("output.json", "w") as f:
            json.dump([], f)
        print("No benchmark results found.", file=sys.stderr)
        sys.exit(1)

    # Write JSON for benchmark-action
    with open("output.json", "w") as f:
        json.dump(to_benchmark_action_json(results), f, indent=2)

    # Print Markdown to stdout
    print(to_markdown(results))


if __name__ == "__main__":
    main()

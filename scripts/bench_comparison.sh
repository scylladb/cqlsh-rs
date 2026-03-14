#!/usr/bin/env bash
# Compare cqlsh-rs vs Python cqlsh startup using hyperfine.
#
# Usage:
#   scripts/bench_comparison.sh
#
# Environment variables:
#   CQLSH_RS  — path to cqlsh-rs binary (default: target/release/cqlsh-rs)
#   CQLSH_PY  — path to Python cqlsh binary (default: cqlsh)
#   OUTPUT    — markdown output file (default: startup-comparison.md)

set -euo pipefail

CQLSH_RS="${CQLSH_RS:-target/release/cqlsh-rs}"
CQLSH_PY="${CQLSH_PY:-cqlsh}"
OUTPUT="${OUTPUT:-startup-comparison.md}"

# Verify binaries exist
if ! command -v "$CQLSH_RS" &>/dev/null && [ ! -x "$CQLSH_RS" ]; then
    echo "Error: cqlsh-rs not found at '$CQLSH_RS'" >&2
    echo "Build with: cargo build --release" >&2
    exit 1
fi

if ! command -v "$CQLSH_PY" &>/dev/null; then
    echo "Error: Python cqlsh not found at '$CQLSH_PY'" >&2
    echo "Install with: pip install cqlsh" >&2
    exit 1
fi

if ! command -v hyperfine &>/dev/null; then
    echo "Error: hyperfine not installed" >&2
    echo "Install with: sudo apt-get install -y hyperfine" >&2
    exit 1
fi

echo "Comparing startup: $CQLSH_RS vs $CQLSH_PY"

hyperfine \
    --warmup 3 \
    --min-runs 10 \
    --export-markdown "$OUTPUT" \
    --command-name "cqlsh-rs" "$CQLSH_RS --version" \
    --command-name "python-cqlsh" "$CQLSH_PY --version"

echo "Results written to $OUTPUT"

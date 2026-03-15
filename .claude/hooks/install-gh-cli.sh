#!/usr/bin/env bash
# SessionStart hook: install gh CLI if not present
set -euo pipefail

if command -v gh &>/dev/null; then
  exit 0
fi

GH_VERSION="2.65.0"
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  ARCH="amd64" ;;
  aarch64) ARCH="arm64" ;;
esac

TARBALL="gh_${GH_VERSION}_linux_${ARCH}.tar.gz"
URL="https://github.com/cli/cli/releases/download/v${GH_VERSION}/${TARBALL}"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "$URL" -o "$TMPDIR/$TARBALL"
tar -xzf "$TMPDIR/$TARBALL" -C "$TMPDIR"
cp "$TMPDIR/gh_${GH_VERSION}_linux_${ARCH}/bin/gh" /usr/local/bin/gh
chmod +x /usr/local/bin/gh

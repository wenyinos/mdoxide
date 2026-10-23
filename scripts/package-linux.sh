#!/usr/bin/env bash
# Build Linux release packages (RPM + DEB) for MDOxide.
#
# Prerequisites:
#   cargo install cargo-deb cargo-generate-rpm
#
# Usage: ./scripts/package-linux.sh [--skip-build]
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ "${1:-}" != "--skip-build" ]]; then
    echo "==> cargo build --release"
    cargo build --release
fi

BIN="target/release/mdoxide"
[[ -x "$BIN" ]] || { echo "missing $BIN" >&2; exit 1; }

mkdir -p dist

echo "==> deb"
if command -v cargo-deb >/dev/null; then
    cargo deb --no-build
    cp target/debian/*.deb dist/
else
    echo "cargo-deb not installed (cargo install cargo-deb)" >&2
fi

echo "==> rpm"
if command -v cargo-generate-rpm >/dev/null; then
    cargo generate-rpm
    cp target/generate-rpm/*.rpm dist/
else
    echo "cargo-generate-rpm not installed (cargo install cargo-generate-rpm)" >&2
fi

ls -lh dist/

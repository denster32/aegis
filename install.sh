#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"
cargo install --path crates/aegis --force
echo "Installed: $(command -v aegis)"
aegis --version
aegis auth status || true
echo "Tip: run 'grok login' if auth status fails."

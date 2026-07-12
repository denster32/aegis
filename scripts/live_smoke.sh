#!/usr/bin/env bash
# Live smoke tests for Aegis (requires Grok OAuth or valid API access).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
AEGIS="${AEGIS_BIN:-$HOME/.cargo/bin/aegis}"
# Prefer OAuth over a spent console key (same policy as stress_test.sh)
unset XAI_API_KEY SPACEXAI_API_KEY || true
if [[ ! -x "$AEGIS" ]]; then
  AEGIS="$ROOT/target/release/aegis"
fi
if [[ ! -x "$AEGIS" ]]; then
  echo "build release first: cargo build --release -p aegis && ./install.sh"
  exit 1
fi

echo "== auth status =="
"$AEGIS" auth status

echo "== pong =="
out="$("$AEGIS" -p "Reply with exactly: pong" --effort low 2>&1)"
echo "$out"
echo "$out" | grep -qi pong

echo "== tool write =="
rm -rf /tmp/aegis-smoke
mkdir -p /tmp/aegis-smoke
"$AEGIS" --yolo --cwd /tmp/aegis-smoke -p "Create a file named hello.txt containing exactly the word hi and nothing else." --effort low 2>&1 | tail -20
test -f /tmp/aegis-smoke/hello.txt
grep -q hi /tmp/aegis-smoke/hello.txt

echo "== session list =="
"$AEGIS" session list --limit 3

echo "ALL SMOKES PASSED"

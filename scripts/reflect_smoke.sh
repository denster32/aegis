#!/usr/bin/env bash
set -euo pipefail
AEGIS="${AEGIS_BIN:-$HOME/.cargo/bin/aegis}"
DIR=$(mktemp -d /tmp/aegis-learn-XXXX)
cleanup() { rm -rf "$DIR"; }
trap cleanup EXIT
cp -r "$(dirname "$0")/../examples/toy-project/." "$DIR/"
cd "$DIR"
echo "== first run (seed memory) =="
"$AEGIS" --yolo --cwd "$DIR" -p "Read the crate, run tests with cargo test, then use memory_write to record a lesson that tests must be run from crate root with cargo test." --effort low 2>&1 | tail -25
echo "== memory =="
"$AEGIS" --cwd "$DIR" memory show
test -f "$DIR/.aegis/MEMORY.md" || test -f "$DIR/.aegis/LESSONS.jsonl"
echo "REFLECT SMOKE OK"

#!/usr/bin/env bash
# Pack a privacy-preserving Aegis spore from the current project.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:-$ROOT/.aegis/nexus/spore-out}"
AEGIS="${AEGIS_BIN:-$HOME/.cargo/bin/aegis}"
"$AEGIS" --cwd "$ROOT" spore pack --out "$OUT"
echo "Spore ready: $OUT"
echo "Vaccinate elsewhere: aegis spore vaccinate $OUT"

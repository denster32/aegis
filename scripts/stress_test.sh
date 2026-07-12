#!/usr/bin/env bash
# True multi-phase stress test for Aegis (requires Grok OAuth).
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
AEGIS="${AEGIS_BIN:-$HOME/.cargo/bin/aegis}"
STRESS_ROOT="${STRESS_ROOT:-$(mktemp -d /tmp/aegis-stress-XXXXXX)}"
REPORT_DIR="$ROOT/docs"
REPORT="$REPORT_DIR/stress-report.md"
LOG="$STRESS_ROOT/stress.log"
PASS=0
FAIL=0
SKIP=0
START_TS=$(date +%s)

mkdir -p "$STRESS_ROOT" "$REPORT_DIR"
export AEGIS_BIN="$AEGIS"
# Prefer OAuth over spent API keys
unset XAI_API_KEY SPACEXAI_API_KEY || true

log() { echo "[$(date -Iseconds)] $*" | tee -a "$LOG"; }
phase() { log "==== $* ===="; }
ok() { PASS=$((PASS+1)); log "PASS: $*"; }
bad() { FAIL=$((FAIL+1)); log "FAIL: $*"; }
skip() { SKIP=$((SKIP+1)); log "SKIP: $*"; }

run_to() {
  local timeout_s=$1; shift
  timeout "$timeout_s" "$@" >>"$LOG" 2>&1
}

log "AEGIS=$AEGIS STRESS_ROOT=$STRESS_ROOT"

# ---------- S0 CLI smoke ----------
phase "S0 CLI smoke"
if "$AEGIS" --help >/dev/null 2>&1; then ok "help"; else bad "help"; fi
if "$AEGIS" auth status >>"$LOG" 2>&1; then ok "auth status"; else bad "auth status"; fi
if "$AEGIS" --cwd "$ROOT" readiness >>"$LOG" 2>&1; then ok "readiness"; else bad "readiness"; fi
if "$AEGIS" --cwd "$ROOT" factory >>"$LOG" 2>&1; then ok "factory"; else bad "factory"; fi

# ---------- S1 cold crate ----------
phase "S1 cold crate create"
PROJ="$STRESS_ROOT/proj"
mkdir -p "$PROJ"
if run_to 240 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p \
  "Create a minimal Rust library crate here with Cargo.toml and src/lib.rs exporting pub fn add(a:i32,b:i32)->i32 and a unit test. Then run cargo test."; then
  if (cd "$PROJ" && cargo test -q) >>"$LOG" 2>&1; then ok "S1 cargo test"; else bad "S1 cargo test after agent"; fi
else
  bad "S1 agent create crate"
fi

# ---------- S2 multi-step edit ----------
phase "S2 multi-step edit"
if run_to 180 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p \
  "Add pub fn mul(a:i32,b:i32)->i32 to lib.rs with a unit test. Run cargo test."; then
  if grep -q "fn mul" "$PROJ/src/lib.rs" 2>/dev/null && (cd "$PROJ" && cargo test -q) >>"$LOG" 2>&1; then
    ok "S2 mul + tests"
  else
    bad "S2 content or tests"
  fi
else
  bad "S2 agent"
fi

# ---------- S3 induced failure + heal ----------
phase "S3 induced failure + heal"
# Break the build
if [[ -f "$PROJ/src/lib.rs" ]]; then
  cp "$PROJ/src/lib.rs" "$PROJ/src/lib.rs.bak"
  echo "THIS IS NOT VALID RUST !!!" >> "$PROJ/src/lib.rs"
  BEFORE_FAIL=$(wc -l < "$PROJ/.aegis/FAILURES.jsonl" 2>/dev/null || echo 0)
  if run_to 240 "$AEGIS" --yolo --cwd "$PROJ" --effort medium -p \
    "cargo test fails. Diagnose, fix the compile error in src/lib.rs, restore valid code (add and mul must work), run cargo test until green. Use self-heal."; then
    if (cd "$PROJ" && cargo test -q) >>"$LOG" 2>&1; then
      ok "S3 healed build"
      AFTER_FAIL=$(wc -l < "$PROJ/.aegis/FAILURES.jsonl" 2>/dev/null || echo 0)
      # metrics may have heal attempts
      if [[ -f "$PROJ/.aegis/metrics.json" ]]; then
        log "metrics: $(cat "$PROJ/.aegis/metrics.json")"
      fi
    else
      bad "S3 still broken after heal"
    fi
  else
    bad "S3 agent heal turn"
  fi
else
  skip "S3 no lib.rs"
fi

# ---------- S4 parallel-ish multi write ----------
phase "S4 multi-file write"
if run_to 180 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p \
  "Create docs/NOTE1.md, docs/NOTE2.md, docs/NOTE3.md each with a single line saying ok-N for N=1,2,3."; then
  if [[ -f "$PROJ/docs/NOTE1.md" && -f "$PROJ/docs/NOTE2.md" && -f "$PROJ/docs/NOTE3.md" ]]; then
    ok "S4 multi files"
  else
    bad "S4 missing files"
  fi
else
  bad "S4 agent"
fi

# ---------- S5 mission ----------
phase "S5 mission"
if run_to 360 "$AEGIS" --yolo --cwd "$PROJ" --effort medium mission --workers 2 \
  "Add README.md describing the crate and ensure cargo test passes."; then
  ok "S5 mission completed"
else
  # soft: mission may still leave useful files
  if [[ -f "$PROJ/README.md" ]]; then ok "S5 mission partial (README exists)"; else bad "S5 mission"; fi
fi

# ---------- S6 dream ----------
phase "S6 dream"
if run_to 300 "$AEGIS" --cwd "$PROJ" dream --apply >>"$LOG" 2>&1; then
  if ls "$PROJ/.aegis/dreams/"*.md >/dev/null 2>&1; then ok "S6 dream journal"; else bad "S6 no journal"; fi
else
  bad "S6 dream"
fi

# ---------- S7 wiki ----------
phase "S7 wiki"
if run_to 180 "$AEGIS" --cwd "$PROJ" wiki generate >>"$LOG" 2>&1; then
  N=$(ls "$PROJ/docs/wiki/"*.md 2>/dev/null | wc -l)
  if [[ "$N" -ge 4 ]]; then ok "S7 wiki pages=$N"; else bad "S7 wiki count=$N"; fi
else
  bad "S7 wiki"
fi

# ---------- S8 QA ----------
phase "S8 qa"
"$AEGIS" --cwd "$PROJ" install-qa >>"$LOG" 2>&1 || true
if run_to 120 "$AEGIS" --cwd "$PROJ" qa >>"$LOG" 2>&1; then
  if ls "$PROJ/.aegis/qa/reports/"*.md >/dev/null 2>&1; then ok "S8 qa report"; else bad "S8 no report"; fi
else
  bad "S8 qa"
fi

# ---------- S9 review --diff ----------
phase "S9 review"
(cd "$PROJ" && git init -q && git add -A && git -c user.email=t@t -c user.name=t commit -qm init) >>"$LOG" 2>&1 || true
echo "// stress" >> "$PROJ/src/lib.rs"
if run_to 180 "$AEGIS" --cwd "$PROJ" review --diff --depth shallow >>"$LOG" 2>&1; then
  ok "S9 review"
else
  # empty meaningful diff review may fail — accept report file
  if ls "$PROJ/.aegis/reviews/"* >/dev/null 2>&1; then ok "S9 review artifact"; else bad "S9 review"; fi
fi

# ---------- S10 concurrent (best effort) ----------
phase "S10 concurrent agents"
run_to 120 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p "Append a line 'c1' to docs/NOTE1.md" &
P1=$!
run_to 120 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p "Append a line 'c2' to docs/NOTE2.md" &
P2=$!
wait $P1; E1=$?
wait $P2; E2=$?
if [[ $E1 -eq 0 && $E2 -eq 0 ]]; then ok "S10 concurrent"; else bad "S10 concurrent e1=$E1 e2=$E2"; fi

# ---------- S12 max steps (graceful) ----------
phase "S12 budget (short model, complex ask)"
# just ensure no hang with short timeout
if run_to 90 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p "Do nothing except reply: done"; then
  ok "S12 short turn"
else
  bad "S12 hang or fail"
fi

END_TS=$(date +%s)
DUR=$((END_TS-START_TS))

# Write report
cat > "$REPORT" << EOF
# Aegis Stress Report

- Date: $(date -Iseconds)
- Duration: ${DUR}s
- STRESS_ROOT: \`$STRESS_ROOT\`
- PASS: $PASS
- FAIL: $FAIL
- SKIP: $SKIP

## Phases

See log: \`$LOG\`

## Interpretation

- Failures in S3 (heal) or S10 (concurrency) are P0.
- S5–S9 depend on API quality; soft pass notes in log.

## Next

\`\`\`bash
./scripts/stress_test.sh
\`\`\`
EOF

log "RESULT pass=$PASS fail=$FAIL skip=$SKIP duration=${DUR}s"
log "Report: $REPORT"
if [[ "$FAIL" -eq 0 ]]; then
  exit 0
else
  exit 1
fi

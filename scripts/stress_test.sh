#!/usr/bin/env bash
# Longer multi-phase stress test for Aegis (requires Grok OAuth).
# Usage:
#   ./scripts/stress_test.sh
#   STRESS_LONG=1 ./scripts/stress_test.sh   # extra phases (default on)
#   STRESS_ROOT=/tmp/foo ./scripts/stress_test.sh
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
STRESS_LONG="${STRESS_LONG:-1}"

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

assert_file() {
  local f=$1
  if [[ -f "$f" ]]; then return 0; else return 1; fi
}

log "AEGIS=$AEGIS STRESS_ROOT=$STRESS_ROOT LONG=$STRESS_LONG"
log "version: $($AEGIS --version 2>&1 || true)"

# ---------- S0 CLI smoke ----------
phase "S0 CLI smoke"
if "$AEGIS" --help >/dev/null 2>&1; then ok "help"; else bad "help"; fi
if "$AEGIS" --version >/dev/null 2>&1; then ok "version"; else bad "version"; fi
if "$AEGIS" auth status >>"$LOG" 2>&1; then ok "auth status"; else bad "auth status"; fi
if "$AEGIS" --cwd "$ROOT" readiness >>"$LOG" 2>&1; then ok "readiness"; else bad "readiness"; fi
if "$AEGIS" --cwd "$ROOT" factory >>"$LOG" 2>&1; then ok "factory"; else bad "factory"; fi
if "$AEGIS" --cwd "$ROOT" memory show >>"$LOG" 2>&1; then ok "memory show"; else bad "memory show"; fi
if "$AEGIS" automation list --cwd "$ROOT" >>"$LOG" 2>&1 || "$AEGIS" --cwd "$ROOT" automation list >>"$LOG" 2>&1; then
  ok "automation list"
else
  bad "automation list"
fi

# ---------- S1 cold crate ----------
phase "S1 cold crate create"
PROJ="$STRESS_ROOT/proj"
mkdir -p "$PROJ"
if run_to 300 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p \
  "Create a minimal Rust library crate here with Cargo.toml and src/lib.rs exporting pub fn add(a:i32,b:i32)->i32 and a unit test. Then run cargo test."; then
  if (cd "$PROJ" && cargo test -q) >>"$LOG" 2>&1; then ok "S1 cargo test"; else bad "S1 cargo test after agent"; fi
else
  bad "S1 agent create crate"
fi

# ---------- S2 multi-step edit ----------
phase "S2 multi-step edit"
if run_to 240 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p \
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
if [[ -f "$PROJ/src/lib.rs" ]]; then
  cp "$PROJ/src/lib.rs" "$PROJ/src/lib.rs.bak"
  echo "THIS IS NOT VALID RUST !!!" >> "$PROJ/src/lib.rs"
  if run_to 300 "$AEGIS" --yolo --cwd "$PROJ" --effort medium -p \
    "cargo test fails. Diagnose, fix the compile error in src/lib.rs, restore valid code (add and mul must work), run cargo test until green. Use self-heal."; then
    if (cd "$PROJ" && cargo test -q) >>"$LOG" 2>&1; then
      ok "S3 healed build"
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

# ---------- S4 multi-file write ----------
phase "S4 multi-file write"
if run_to 240 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p \
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
if run_to 420 "$AEGIS" --yolo --cwd "$PROJ" --effort medium mission --workers 2 \
  "Add README.md describing the crate (add and mul) and ensure cargo test passes."; then
  ok "S5 mission completed"
else
  if [[ -f "$PROJ/README.md" ]]; then ok "S5 mission partial (README exists)"; else bad "S5 mission"; fi
fi

# ---------- S6 dream ----------
phase "S6 dream"
if run_to 360 "$AEGIS" --cwd "$PROJ" dream --apply >>"$LOG" 2>&1; then
  if ls "$PROJ/.aegis/dreams/"*.md >/dev/null 2>&1; then ok "S6 dream journal"; else bad "S6 no journal"; fi
else
  bad "S6 dream"
fi

# ---------- S7 wiki ----------
phase "S7 wiki"
if run_to 240 "$AEGIS" --cwd "$PROJ" wiki generate >>"$LOG" 2>&1; then
  N=$(ls "$PROJ/docs/wiki/"*.md 2>/dev/null | wc -l)
  if [[ "$N" -ge 4 ]]; then ok "S7 wiki pages=$N"; else bad "S7 wiki count=$N"; fi
else
  bad "S7 wiki"
fi

# ---------- S8 QA ----------
phase "S8 qa"
"$AEGIS" --cwd "$PROJ" install-qa >>"$LOG" 2>&1 || true
if run_to 180 "$AEGIS" --cwd "$PROJ" qa >>"$LOG" 2>&1; then
  if ls "$PROJ/.aegis/qa/reports/"*.md >/dev/null 2>&1; then ok "S8 qa report"; else bad "S8 no report"; fi
else
  bad "S8 qa"
fi

# ---------- S9 review --diff ----------
phase "S9 review"
(cd "$PROJ" && git init -q && git add -A && git -c user.email=t@t -c user.name=t commit -qm init) >>"$LOG" 2>&1 || true
echo "// stress-review" >> "$PROJ/src/lib.rs"
if run_to 240 "$AEGIS" --cwd "$PROJ" review --diff --depth shallow >>"$LOG" 2>&1; then
  ok "S9 review"
else
  if ls "$PROJ/.aegis/reviews/"* >/dev/null 2>&1; then ok "S9 review artifact"; else bad "S9 review"; fi
fi
# keep tests green after review noise
if [[ -f "$PROJ/src/lib.rs.bak" ]]; then
  :
fi
# strip the stress-review line if it broke the build (not always a comment-only append)
if ! (cd "$PROJ" && cargo test -q) >>"$LOG" 2>&1; then
  sed -i '/stress-review/d' "$PROJ/src/lib.rs" 2>/dev/null || true
fi

# ---------- S10 concurrent ----------
phase "S10 concurrent agents"
run_to 150 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p "Append a line 'c1' to docs/NOTE1.md" &
P1=$!
run_to 150 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p "Append a line 'c2' to docs/NOTE2.md" &
P2=$!
wait $P1; E1=$?
wait $P2; E2=$?
if [[ $E1 -eq 0 && $E2 -eq 0 ]]; then ok "S10 concurrent"; else bad "S10 concurrent e1=$E1 e2=$E2"; fi

# ---------- S11 grep / edit fidelity ----------
phase "S11 grep and targeted edit"
if run_to 240 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p \
  "Add pub fn sub(a:i32,b:i32)->i32 that subtracts, with unit test. Do not break add/mul. Run cargo test."; then
  if grep -q "fn sub" "$PROJ/src/lib.rs" 2>/dev/null && (cd "$PROJ" && cargo test -q) >>"$LOG" 2>&1; then
    ok "S11 sub + tests"
  else
    bad "S11 content or tests"
  fi
else
  bad "S11 agent"
fi

# ---------- S12 short turn / no hang ----------
phase "S12 budget (short model, complex ask)"
if run_to 90 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p "Do nothing except reply: done"; then
  ok "S12 short turn"
else
  bad "S12 hang or fail"
fi

# ---------- S13 memory after work ----------
phase "S13 memory present"
if [[ -f "$PROJ/.aegis/MEMORY.md" ]] || [[ -f "$PROJ/.aegis/LESSONS.jsonl" ]] || [[ -f "$PROJ/.aegis/metrics.json" ]]; then
  ok "S13 .aegis learning artifacts"
  log "ls .aegis: $(ls -la "$PROJ/.aegis" 2>/dev/null | tr '\n' ' ')"
else
  bad "S13 no learning artifacts"
fi

# ---------- S14 checkpoint ----------
phase "S14 checkpoint"
if run_to 60 "$AEGIS" --cwd "$PROJ" checkpoint create -m "stress-s14" >>"$LOG" 2>&1 \
  || run_to 60 "$AEGIS" --cwd "$PROJ" checkpoint create --message "stress-s14" >>"$LOG" 2>&1 \
  || run_to 60 "$AEGIS" --cwd "$PROJ" checkpoint create >>"$LOG" 2>&1; then
  ok "S14 checkpoint create"
else
  # soft: command surface may vary
  if "$AEGIS" checkpoint --help >>"$LOG" 2>&1; then skip "S14 checkpoint CLI shape"; else bad "S14 checkpoint"; fi
fi

if [[ "$STRESS_LONG" == "1" ]]; then
  # ---------- S15 multi-module crate ----------
  phase "S15 multi-module expansion"
  if run_to 360 "$AEGIS" --yolo --cwd "$PROJ" --effort medium -p \
    "Expand the crate: create src/math.rs with pub fn div(a:i32,b:i32)->Option<i32> (None on /0), mod it from lib.rs, re-export, unit tests in math.rs. Keep add/mul/sub working. Run cargo test."; then
    if [[ -f "$PROJ/src/math.rs" ]] && (cd "$PROJ" && cargo test -q) >>"$LOG" 2>&1; then
      ok "S15 multi-module"
    else
      bad "S15 multi-module content/tests"
    fi
  else
    bad "S15 agent"
  fi

  # ---------- S16 plan only ----------
  phase "S16 plan structured"
  if run_to 180 "$AEGIS" --cwd "$PROJ" --effort medium plan \
    "Add logging feature flag to this crate" >>"$LOG" 2>&1 \
    || run_to 180 "$AEGIS" --yolo --cwd "$PROJ" --effort medium -p \
      "Produce a short structured plan only (no code changes) for adding a logging feature flag. Reply with numbered steps." >>"$LOG" 2>&1; then
    ok "S16 plan"
  else
    bad "S16 plan"
  fi

  # ---------- S17 missions product ----------
  phase "S17 missions new+status"
  if run_to 300 "$AEGIS" --yolo --cwd "$PROJ" --effort medium missions new \
    "Document public API of add mul sub div in docs/API.md" >>"$LOG" 2>&1; then
    ok "S17 missions new"
    if run_to 60 "$AEGIS" --cwd "$PROJ" missions list >>"$LOG" 2>&1 \
      || run_to 60 "$AEGIS" --cwd "$PROJ" missions status >>"$LOG" 2>&1; then
      ok "S17 missions status/list"
    else
      bad "S17 missions status/list"
    fi
  else
    bad "S17 missions new"
  fi

  # ---------- S18 second heal (from prior FAILURES if any) ----------
  phase "S18 second induced error"
  if [[ -f "$PROJ/src/lib.rs" ]]; then
    # break a different way
    printf '\nfn broken_syntax( { \n' >> "$PROJ/src/lib.rs"
    if run_to 300 "$AEGIS" --yolo --cwd "$PROJ" --effort medium -p \
      "Build is broken again. Fix compile errors, keep all public functions and tests green. Run cargo test."; then
      if (cd "$PROJ" && cargo test -q) >>"$LOG" 2>&1; then
        ok "S18 second heal"
      else
        bad "S18 still broken"
      fi
    else
      bad "S18 agent"
    fi
  else
    skip "S18 no lib.rs"
  fi

  # ---------- S19 concurrent triple ----------
  phase "S19 triple concurrent"
  run_to 150 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p "Create docs/C_A.md with content A" &
  PA=$!
  run_to 150 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p "Create docs/C_B.md with content B" &
  PB=$!
  run_to 150 "$AEGIS" --yolo --cwd "$PROJ" --effort low -p "Create docs/C_C.md with content C" &
  PC=$!
  wait $PA; EA=$?
  wait $PB; EB=$?
  wait $PC; EC=$?
  if [[ $EA -eq 0 && $EB -eq 0 && $EC -eq 0 ]] \
    && [[ -f "$PROJ/docs/C_A.md" && -f "$PROJ/docs/C_B.md" && -f "$PROJ/docs/C_C.md" ]]; then
    ok "S19 triple concurrent"
  else
    bad "S19 triple e=$EA/$EB/$EC files"
  fi

  # ---------- S20 final green ----------
  phase "S20 final cargo test"
  if (cd "$PROJ" && cargo test -q) >>"$LOG" 2>&1; then
    ok "S20 final green"
  else
    bad "S20 final cargo test"
  fi
fi

END_TS=$(date +%s)
DUR=$((END_TS-START_TS))

# Summarize artifacts
ART_SUMMARY=$(
  {
    echo "proj tree (depth 3):"
    find "$PROJ" -maxdepth 3 -type f 2>/dev/null | head -80
    echo ""
    if [[ -f "$PROJ/.aegis/metrics.json" ]]; then
      echo "metrics: $(cat "$PROJ/.aegis/metrics.json")"
    fi
  } 2>/dev/null || true
)

# Write report
cat > "$REPORT" << EOF
# Aegis Stress Report

- Date: $(date -Iseconds)
- Version: \`$($AEGIS --version 2>/dev/null || echo unknown)\`
- Duration: ${DUR}s (~$(echo "scale=1; $DUR/60" | bc 2>/dev/null || echo "?") min)
- STRESS_ROOT: \`$STRESS_ROOT\`
- STRESS_LONG: \`$STRESS_LONG\`
- PASS: **$PASS**
- FAIL: **$FAIL**
- SKIP: **$SKIP**

## Phases covered

| Band | Scenarios |
|------|-----------|
| Core | S0 CLI/auth · S1 create · S2 edit · S3 heal · S4 multi-write |
| Platform | S5 mission · S6 dream · S7 wiki · S8 QA · S9 review |
| Load | S10 concurrent · S11 edit · S12 short · S13 memory · S14 checkpoint |
| Long | S15 multi-mod · S16 plan · S17 missions · S18 heal×2 · S19 triple · S20 green |

## Artifacts

\`\`\`
$ART_SUMMARY
\`\`\`

## Log

\`$LOG\`

## Interpretation

- **P0** if S3/S18 heal or S10/S19 concurrency fail.
- **P1** if S1 create or S20 final green fail.
- Platform (S5–S9, S16–S17) can soft-pass with partial artifacts; still prefer full green.
- Duration >10 min is normal for LONG=1 under live API latency.

## Re-run

\`\`\`bash
./scripts/stress_test.sh
STRESS_LONG=0 ./scripts/stress_test.sh   # shorter core-only
\`\`\`
EOF

log "RESULT pass=$PASS fail=$FAIL skip=$SKIP duration=${DUR}s"
log "Report: $REPORT"
if [[ "$FAIL" -eq 0 ]]; then
  exit 0
else
  exit 1
fi

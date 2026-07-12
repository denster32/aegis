# Aegis Stress Report

- Date: 2026-07-11T23:33:28-05:00
- Version: `aegis 0.5.1`
- Duration: 285s (~4.7 min)
- STRESS_ROOT: `/tmp/aegis-stress-ZYkwLG`
- STRESS_LONG: `1`
- PASS: **28**
- FAIL: **0**
- SKIP: **0**

## Phases covered

| Band | Scenarios |
|------|-----------|
| Core | S0 CLI/auth · S1 create · S2 edit · S3 heal · S4 multi-write |
| Platform | S5 mission · S6 dream · S7 wiki · S8 QA · S9 review |
| Load | S10 concurrent · S11 edit · S12 short · S13 memory · S14 checkpoint |
| Long | S15 multi-mod · S16 plan · S17 missions · S18 heal×2 · S19 triple · S20 green |

## Artifacts

```
proj tree (depth 3):
/tmp/aegis-stress-ZYkwLG/proj/Cargo.toml
/tmp/aegis-stress-ZYkwLG/proj/README.md
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/sendemail-validate.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/pre-merge-commit.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/pre-push.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/pre-rebase.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/pre-applypatch.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/push-to-checkout.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/post-update.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/fsmonitor-watchman.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/pre-commit.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/update.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/commit-msg.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/pre-receive.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/prepare-commit-msg.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/hooks/applypatch-msg.sample
/tmp/aegis-stress-ZYkwLG/proj/.git/COMMIT_EDITMSG
/tmp/aegis-stress-ZYkwLG/proj/.git/logs/HEAD
/tmp/aegis-stress-ZYkwLG/proj/.git/AUTO_MERGE
/tmp/aegis-stress-ZYkwLG/proj/.git/refs/stash
/tmp/aegis-stress-ZYkwLG/proj/.git/index
/tmp/aegis-stress-ZYkwLG/proj/.git/info/exclude
/tmp/aegis-stress-ZYkwLG/proj/.git/ORIG_HEAD
/tmp/aegis-stress-ZYkwLG/proj/.git/config
/tmp/aegis-stress-ZYkwLG/proj/.git/HEAD
/tmp/aegis-stress-ZYkwLG/proj/.git/description
/tmp/aegis-stress-ZYkwLG/proj/Cargo.lock
/tmp/aegis-stress-ZYkwLG/proj/src/lib.rs
/tmp/aegis-stress-ZYkwLG/proj/src/lib.rs.bak
/tmp/aegis-stress-ZYkwLG/proj/src/math.rs
/tmp/aegis-stress-ZYkwLG/proj/.github/workflows/aegis-qa.yml
/tmp/aegis-stress-ZYkwLG/proj/docs/C_A.md
/tmp/aegis-stress-ZYkwLG/proj/docs/NOTE2.md
/tmp/aegis-stress-ZYkwLG/proj/docs/C_C.md
/tmp/aegis-stress-ZYkwLG/proj/docs/C_B.md
/tmp/aegis-stress-ZYkwLG/proj/docs/wiki/Architecture.md
/tmp/aegis-stress-ZYkwLG/proj/docs/wiki/Commands.md
/tmp/aegis-stress-ZYkwLG/proj/docs/wiki/Modules.md
/tmp/aegis-stress-ZYkwLG/proj/docs/wiki/Conventions.md
/tmp/aegis-stress-ZYkwLG/proj/docs/wiki/Home.md
/tmp/aegis-stress-ZYkwLG/proj/docs/wiki/_Sidebar.md
/tmp/aegis-stress-ZYkwLG/proj/docs/NOTE1.md
/tmp/aegis-stress-ZYkwLG/proj/docs/NOTE3.md
/tmp/aegis-stress-ZYkwLG/proj/target/.rustc_info.json
/tmp/aegis-stress-ZYkwLG/proj/target/debug/.cargo-build-lock
/tmp/aegis-stress-ZYkwLG/proj/target/debug/.cargo-lock
/tmp/aegis-stress-ZYkwLG/proj/target/debug/.cargo-artifact-lock
/tmp/aegis-stress-ZYkwLG/proj/target/CACHEDIR.TAG
/tmp/aegis-stress-ZYkwLG/proj/.aegis/reviews/diff-20260712_043203.md
/tmp/aegis-stress-ZYkwLG/proj/.aegis/LESSONS.jsonl
/tmp/aegis-stress-ZYkwLG/proj/.aegis/dreams/2026-07-12_043151.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/dreams/2026-07-12_043151.md
/tmp/aegis-stress-ZYkwLG/proj/.aegis/SKILLS/rust-lib-heal.md
/tmp/aegis-stress-ZYkwLG/proj/.aegis/checkpoints/20260712_043242.status.txt
/tmp/aegis-stress-ZYkwLG/proj/.aegis/checkpoints/20260712_043127.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/checkpoints/20260712_043127.status.txt
/tmp/aegis-stress-ZYkwLG/proj/.aegis/checkpoints/20260712_043242.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/metrics.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/FAILURES.jsonl
/tmp/aegis-stress-ZYkwLG/proj/.aegis/readiness/report.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/0f98d554-b4d7-45ff-9c72-bbcd5e5b6a03.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/1c2a4df8-12e6-4ecd-992d-f08b971b8a3a.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/ce2591fa-cc6b-4e96-95f1-47973b1bb2f4.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/4e8fec3e-f41d-435f-b5ca-758b279776df.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/6f40b133-f0db-4bd6-adda-58cf754e7ed9.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/20171d9a-8d1e-4beb-8fe4-e228a9bdc30e.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/94719d03-c05a-4953-84b0-5e26cb9af47e.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/d4806777-c872-4d23-9058-90cb32fd368a.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/a138bf15-a755-444f-b834-fb66ed85f6c5.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/a9f27ba8-d95d-4aeb-a77e-c5f18621edb0.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/e2f1851d-f876-441d-8d4c-dba832719139.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/fb17a86e-ce8f-4976-b02f-6e741dfc25c5.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/2bc01a7a-0bcf-4842-98c2-8f11f1ec5018.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/runs/40d319a1-889b-46ce-9969-92b06e16bc4c.json
/tmp/aegis-stress-ZYkwLG/proj/.aegis/MEMORY.md

metrics: {
  "run_count": 23,
  "heal_attempts": 2,
  "heal_successes": 2,
  "last_run_id": "a9f27ba8-d95d-4aeb-a77e-c5f18621edb0",
  "last_run_at": "2026-07-12T04:33:13.977636996+00:00"
}
```

## Log

`/tmp/aegis-stress-ZYkwLG/stress.log`

## Interpretation

- **P0** if S3/S18 heal or S10/S19 concurrency fail.
- **P1** if S1 create or S20 final green fail.
- Platform (S5–S9, S16–S17) can soft-pass with partial artifacts; still prefer full green.
- Duration >10 min is normal for LONG=1 under live API latency.

## Re-run

```bash
./scripts/stress_test.sh
STRESS_LONG=0 ./scripts/stress_test.sh   # shorter core-only
```

# Aegis Stress Report

- Date: 2026-07-12T00:16:08-05:00
- Version: `aegis 0.7.0`
- Duration: 201s (~3.3 min)
- STRESS_ROOT: `/tmp/aegis-stress-lHjVmf`
- STRESS_LONG: `1`
- PASS: **31**
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
proj highlights:
/tmp/aegis-stress-lHjVmf/proj/Cargo.toml
/tmp/aegis-stress-lHjVmf/proj/README.md
/tmp/aegis-stress-lHjVmf/proj/Cargo.lock
/tmp/aegis-stress-lHjVmf/proj/src/lib.rs
/tmp/aegis-stress-lHjVmf/proj/src/lib.rs.bak
/tmp/aegis-stress-lHjVmf/proj/src/math.rs
/tmp/aegis-stress-lHjVmf/proj/.github/workflows/aegis-qa.yml
/tmp/aegis-stress-lHjVmf/proj/docs/C_A.md
/tmp/aegis-stress-lHjVmf/proj/docs/NOTE2.md
/tmp/aegis-stress-lHjVmf/proj/docs/C_C.md
/tmp/aegis-stress-lHjVmf/proj/docs/C_B.md
/tmp/aegis-stress-lHjVmf/proj/docs/wiki/Architecture.md
/tmp/aegis-stress-lHjVmf/proj/docs/wiki/Commands.md
/tmp/aegis-stress-lHjVmf/proj/docs/wiki/Modules.md
/tmp/aegis-stress-lHjVmf/proj/docs/wiki/Conventions.md
/tmp/aegis-stress-lHjVmf/proj/docs/wiki/Home.md
/tmp/aegis-stress-lHjVmf/proj/docs/wiki/_Sidebar.md
/tmp/aegis-stress-lHjVmf/proj/docs/NOTE1.md
/tmp/aegis-stress-lHjVmf/proj/docs/NOTE3.md
/tmp/aegis-stress-lHjVmf/proj/.aegis/reviews/diff-20260712_051458.md
/tmp/aegis-stress-lHjVmf/proj/.aegis/LESSONS.jsonl
/tmp/aegis-stress-lHjVmf/proj/.aegis/dreams/2026-07-12_051445.md
/tmp/aegis-stress-lHjVmf/proj/.aegis/dreams/2026-07-12_051445.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/SKILLS/proj-minimal-rust-conventions.md
/tmp/aegis-stress-lHjVmf/proj/.aegis/SKILLS/rust-trailing-junk-heal.md
/tmp/aegis-stress-lHjVmf/proj/.aegis/checkpoints/20260712_051525.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/checkpoints/20260712_051525.status.txt
/tmp/aegis-stress-lHjVmf/proj/.aegis/checkpoints/20260712_051422.status.txt
/tmp/aegis-stress-lHjVmf/proj/.aegis/checkpoints/20260712_051422.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/nexus/neural-summary.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/metrics.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/FAILURES.jsonl
/tmp/aegis-stress-lHjVmf/proj/.aegis/readiness/report.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/0cd3e2a2-e757-4bf0-859b-23783f30d575.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/b8eefed0-c982-419d-a1c8-9f99b3ad2b65.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/811849a2-2ccb-426d-b40e-84e625d2db3b.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/b669e0c1-cf85-4a79-8bd0-df0e026df83e.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/2b540fb0-0cf8-49eb-bd40-b8d858566e4a.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/8e25097f-7681-469a-9872-5890595a3975.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/bff3e9b7-8d8d-4f28-805e-fc6bb908f2ed.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/22292cb9-1258-4c6a-a202-9e02f9cc80d8.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/4eb27c43-26a4-456b-b16c-5974932823fc.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/55f1dcc4-2484-427f-8244-5938c2c24ca3.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/1f92a81f-e850-45cd-994e-0ac8e8ade96a.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/6e7125ca-f7df-4a3f-b433-9ae49eb67914.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/9c29fabd-282a-4525-be89-d518057d2e7e.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/runs/674a53a3-e68a-4654-9829-ef77b54a3c4d.json
/tmp/aegis-stress-lHjVmf/proj/.aegis/MEMORY.md

metrics: {
  "run_count": 23,
  "heal_attempts": 3,
  "heal_successes": 0,
  "last_run_id": "811849a2-2ccb-426d-b40e-84e625d2db3b",
  "last_run_at": "2026-07-12T05:16:01.049949815+00:00"
}
```

## Log

`/tmp/aegis-stress-lHjVmf/stress.log`

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

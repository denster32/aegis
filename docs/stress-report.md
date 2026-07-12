# Aegis Stress Report

- Date: 2026-07-12T01:02:14-05:00
- Version: `aegis 0.8.0`
- Duration: 222s (~3.7 min)
- STRESS_ROOT: `/tmp/aegis-stress-tbnK2P`
- STRESS_LONG: `1`
- PASS: **39**
- FAIL: **0**
- SKIP: **0**

## Phases covered

| Band | Scenarios |
|------|-----------|
| Core | S0 CLI/auth · S1 create · S2 edit · S3 heal · S4 multi-write |
| Platform | S5 mission · S6 dream · S7 wiki · S8 QA · S9 review |
| Load | S10 concurrent · S11 edit · S12 short · S13 memory · S14 checkpoint |
| Long | S15 multi-mod · S16 plan · S17 missions · S18 heal×2 · S19 triple · S20 green |
| Hard | S24 sandbox · S25 locks · S26 ssrf · S27 cli |

## Artifacts

```
proj highlights:
/tmp/aegis-stress-tbnK2P/proj/Cargo.toml
/tmp/aegis-stress-tbnK2P/proj/README.md
/tmp/aegis-stress-tbnK2P/proj/Cargo.lock
/tmp/aegis-stress-tbnK2P/proj/src/lib.rs
/tmp/aegis-stress-tbnK2P/proj/src/lib.rs.bak
/tmp/aegis-stress-tbnK2P/proj/src/math.rs
/tmp/aegis-stress-tbnK2P/proj/.github/workflows/aegis-qa.yml
/tmp/aegis-stress-tbnK2P/proj/docs/C_A.md
/tmp/aegis-stress-tbnK2P/proj/docs/NOTE2.md
/tmp/aegis-stress-tbnK2P/proj/docs/C_C.md
/tmp/aegis-stress-tbnK2P/proj/docs/LOCK.txt
/tmp/aegis-stress-tbnK2P/proj/docs/C_B.md
/tmp/aegis-stress-tbnK2P/proj/docs/wiki/Architecture.md
/tmp/aegis-stress-tbnK2P/proj/docs/wiki/Commands.md
/tmp/aegis-stress-tbnK2P/proj/docs/wiki/Modules.md
/tmp/aegis-stress-tbnK2P/proj/docs/wiki/Conventions.md
/tmp/aegis-stress-tbnK2P/proj/docs/wiki/Home.md
/tmp/aegis-stress-tbnK2P/proj/docs/wiki/_Sidebar.md
/tmp/aegis-stress-tbnK2P/proj/docs/NOTE1.md
/tmp/aegis-stress-tbnK2P/proj/docs/NOTE3.md
/tmp/aegis-stress-tbnK2P/proj/.aegis/reviews/diff-20260712_060021.md
/tmp/aegis-stress-tbnK2P/proj/.aegis/LESSONS.jsonl
/tmp/aegis-stress-tbnK2P/proj/.aegis/dreams/2026-07-12_060007.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/dreams/2026-07-12_060007.md
/tmp/aegis-stress-tbnK2P/proj/.aegis/SKILLS/rust-lib-verify.md
/tmp/aegis-stress-tbnK2P/proj/.aegis/checkpoints/20260712_055943.status.txt
/tmp/aegis-stress-tbnK2P/proj/.aegis/checkpoints/20260712_055943.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/checkpoints/20260712_060043.status.txt
/tmp/aegis-stress-tbnK2P/proj/.aegis/checkpoints/20260712_060043.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/nexus/neural-summary.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/metrics.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/FAILURES.jsonl
/tmp/aegis-stress-tbnK2P/proj/.aegis/readiness/report.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/def540e0-d106-46e3-aad8-22ab33fabcd3.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/200150f1-fcf4-4851-a84d-3abde960471f.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/08d67bfd-b39c-412b-b157-e9e74ff6216e.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/39b30411-3d7c-4f7d-875e-75404b15f751.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/b6e7f363-1210-4950-ad4b-b6fa4575d01c.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/3ac12dc7-f9af-46d8-b1dc-92373d1a6a00.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/63a72503-5549-4fe4-bac1-336939a7c79c.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/4266ee22-2d5b-466d-8d02-71943056f7fe.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/f01592e8-35b8-4668-a6ea-4f8dd5901be6.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/771d7530-303b-48c1-95e6-d827e63eb51f.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/ef0159b7-db2c-48c7-a12a-10466a74528b.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/62c57d09-ee71-405f-8dd9-c39b3abd1cd9.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/91f7fb34-e260-4981-bc19-717cdb2500e8.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/789b326c-d94f-4e5d-a481-2dc859a689ff.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/28e8c0ae-9b1c-489d-b32f-6b8eaab84763.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/0cc5225e-6b9f-4454-ac07-7a2c946456f3.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/23db539b-2030-4da6-8048-f01f6af5fab2.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/eb31cd36-969b-4fa9-8e82-43a211bf74b0.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/runs/590f7ee8-6847-4f6c-abb3-6a423101b960.json
/tmp/aegis-stress-tbnK2P/proj/.aegis/MEMORY.md

metrics: {
  "run_count": 28,
  "heal_attempts": 5,
  "heal_successes": 2,
  "last_run_id": "23db539b-2030-4da6-8048-f01f6af5fab2",
  "last_run_at": "2026-07-12T06:02:07.749033997+00:00"
}
```

## Log

`/tmp/aegis-stress-tbnK2P/stress.log`

## Interpretation

- **P0** if S3/S18 heal or S10/S19 concurrency fail.
- **P1** if S1 create or S20 final green fail.
- Hard-fail policy: no soft-pass; every phase needs full criteria.
- Duration >10 min is normal for LONG=1 under live API latency.

## Re-run

```bash
./scripts/stress_test.sh
STRESS_LONG=0 ./scripts/stress_test.sh   # shorter core-only
```

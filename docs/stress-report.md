# Aegis Stress Report

- Date: 2026-07-12T00:59:59-05:00
- Version: `aegis 0.8.0`
- Duration: 232s (~3.8 min)
- STRESS_ROOT: `/tmp/aegis-stress-Z6kGvZ`
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
/tmp/aegis-stress-Z6kGvZ/proj/Cargo.toml
/tmp/aegis-stress-Z6kGvZ/proj/README.md
/tmp/aegis-stress-Z6kGvZ/proj/Cargo.lock
/tmp/aegis-stress-Z6kGvZ/proj/src/lib.rs
/tmp/aegis-stress-Z6kGvZ/proj/src/lib.rs.bak
/tmp/aegis-stress-Z6kGvZ/proj/src/math.rs
/tmp/aegis-stress-Z6kGvZ/proj/.github/workflows/aegis-qa.yml
/tmp/aegis-stress-Z6kGvZ/proj/docs/C_A.md
/tmp/aegis-stress-Z6kGvZ/proj/docs/NOTE2.md
/tmp/aegis-stress-Z6kGvZ/proj/docs/C_C.md
/tmp/aegis-stress-Z6kGvZ/proj/docs/LOCK.txt
/tmp/aegis-stress-Z6kGvZ/proj/docs/C_B.md
/tmp/aegis-stress-Z6kGvZ/proj/docs/wiki/Architecture.md
/tmp/aegis-stress-Z6kGvZ/proj/docs/wiki/Commands.md
/tmp/aegis-stress-Z6kGvZ/proj/docs/wiki/Modules.md
/tmp/aegis-stress-Z6kGvZ/proj/docs/wiki/Conventions.md
/tmp/aegis-stress-Z6kGvZ/proj/docs/wiki/Home.md
/tmp/aegis-stress-Z6kGvZ/proj/docs/wiki/_Sidebar.md
/tmp/aegis-stress-Z6kGvZ/proj/docs/NOTE1.md
/tmp/aegis-stress-Z6kGvZ/proj/docs/NOTE3.md
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/reviews/diff-20260712_055822.md
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/LESSONS.jsonl
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/dreams/2026-07-12_055807.md
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/dreams/2026-07-12_055807.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/SKILLS/rust-crate-module-expand.md
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/SKILLS/rust-self-heal-trailing-junk.md
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/SKILLS/rust-minimal-lib-workflow.md
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/checkpoints/20260712_055845.status.txt
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/checkpoints/20260712_055745.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/checkpoints/20260712_055745.status.txt
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/checkpoints/20260712_055845.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/nexus/neural-summary.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/metrics.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/FAILURES.jsonl
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/readiness/report.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/f1d3d45c-1bd2-43fe-bc37-b21455474da2.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/18d3e7c2-599b-489f-aa01-bff62c0ca2b8.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/d99d36f9-1fb5-42b5-88d4-566e987b6d8c.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/9fe334f8-5875-49fd-9780-cded558bd430.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/c02a52d8-55d3-4f13-ae33-8709aa8f8eff.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/64bb634b-2e31-43da-ba4d-56454530d232.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/d21adf9d-cd66-4c38-bea3-56dcbc0c16ee.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/2ba94732-ce38-470d-8eaf-e9e03db64936.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/cad0f8ae-cef5-49ea-ad07-75fead842d61.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/9b13e98f-bff7-4342-b371-83cf05d9e824.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/ec462cda-4dbd-46f2-be0e-a7c024fc1a35.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/4bad622a-3ac1-485d-9b51-56a41a887f8b.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/b9d0c8d1-1e7d-4783-8ae5-c4a83b68c831.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/58813a11-3d5e-41a6-a997-62da8393de56.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/4de56208-a3ec-4986-b166-2c3af8ecf65f.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/57e72fcc-28fc-4473-9ba2-ce7b20b2d040.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/95b758bd-34f8-4bde-8b36-28c935e96abd.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/e81feaad-099c-4a0a-9d53-9455906808b4.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/runs/abf8c5b5-f037-4528-a05b-0fffa2a023af.json
/tmp/aegis-stress-Z6kGvZ/proj/.aegis/MEMORY.md

metrics: {
  "run_count": 28,
  "heal_attempts": 5,
  "heal_successes": 2,
  "last_run_id": "4bad622a-3ac1-485d-9b51-56a41a887f8b",
  "last_run_at": "2026-07-12T05:59:53.303155904+00:00"
}
```

## Log

`/tmp/aegis-stress-Z6kGvZ/stress.log`

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

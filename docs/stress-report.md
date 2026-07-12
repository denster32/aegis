# Aegis Stress Report

- Date: 2026-07-12T00:47:18-05:00
- Version: `aegis 0.8.0`
- Duration: 224s (~3.7 min)
- STRESS_ROOT: `/tmp/aegis-stress-tkp5It`
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
/tmp/aegis-stress-tkp5It/proj/Cargo.toml
/tmp/aegis-stress-tkp5It/proj/README.md
/tmp/aegis-stress-tkp5It/proj/Cargo.lock
/tmp/aegis-stress-tkp5It/proj/src/lib.rs
/tmp/aegis-stress-tkp5It/proj/src/lib.rs.bak
/tmp/aegis-stress-tkp5It/proj/src/math.rs
/tmp/aegis-stress-tkp5It/proj/.github/workflows/aegis-qa.yml
/tmp/aegis-stress-tkp5It/proj/docs/C_A.md
/tmp/aegis-stress-tkp5It/proj/docs/NOTE2.md
/tmp/aegis-stress-tkp5It/proj/docs/C_C.md
/tmp/aegis-stress-tkp5It/proj/docs/LOCK.txt
/tmp/aegis-stress-tkp5It/proj/docs/C_B.md
/tmp/aegis-stress-tkp5It/proj/docs/wiki/Architecture.md
/tmp/aegis-stress-tkp5It/proj/docs/wiki/Commands.md
/tmp/aegis-stress-tkp5It/proj/docs/wiki/Modules.md
/tmp/aegis-stress-tkp5It/proj/docs/wiki/Conventions.md
/tmp/aegis-stress-tkp5It/proj/docs/wiki/Home.md
/tmp/aegis-stress-tkp5It/proj/docs/wiki/_Sidebar.md
/tmp/aegis-stress-tkp5It/proj/docs/NOTE1.md
/tmp/aegis-stress-tkp5It/proj/docs/NOTE3.md
/tmp/aegis-stress-tkp5It/proj/.aegis/reviews/diff-20260712_054543.md
/tmp/aegis-stress-tkp5It/proj/.aegis/LESSONS.jsonl
/tmp/aegis-stress-tkp5It/proj/.aegis/dreams/2026-07-12_054531.md
/tmp/aegis-stress-tkp5It/proj/.aegis/dreams/2026-07-12_054531.json
/tmp/aegis-stress-tkp5It/proj/.aegis/SKILLS/rust-lib-heal.md
/tmp/aegis-stress-tkp5It/proj/.aegis/checkpoints/20260712_054510.json
/tmp/aegis-stress-tkp5It/proj/.aegis/checkpoints/20260712_054510.status.txt
/tmp/aegis-stress-tkp5It/proj/.aegis/checkpoints/20260712_054606.json
/tmp/aegis-stress-tkp5It/proj/.aegis/checkpoints/20260712_054606.status.txt
/tmp/aegis-stress-tkp5It/proj/.aegis/nexus/neural-summary.json
/tmp/aegis-stress-tkp5It/proj/.aegis/metrics.json
/tmp/aegis-stress-tkp5It/proj/.aegis/FAILURES.jsonl
/tmp/aegis-stress-tkp5It/proj/.aegis/readiness/report.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/b1c0b494-afa4-41de-81da-ec3d78d6cb78.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/45940f5a-8f0f-4eca-a69f-0a387dcd0628.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/517f6f9e-19fb-4d02-ae81-d9105e7979fc.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/c57cce22-acc0-4347-bfee-df238d848701.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/c52488f6-0334-4c48-addc-0129b3b35ce6.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/01c0d935-9d5d-4f40-af5c-c69bc1503da9.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/2fe2675c-380a-4d32-b9d7-909eeacd3199.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/c1296f53-f353-4fa9-8fe0-e69e99a966f1.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/523fc977-23ca-4cc2-9cf9-5097adbb2479.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/87b18f22-1e2c-4aac-b032-867e769b58a2.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/fe00377d-4366-465b-8fb5-bbc352e9e52f.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/42877f8f-7d7e-4fc7-b0a0-aac6e8bd4f48.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/503a8471-3c37-413e-9594-e156dc25ef1c.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/2b3d9f4c-9e89-44b0-a2c3-83d217863d21.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/58020678-f253-4a4f-a4ba-0e5e88f9e6cd.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/2a65e31b-84fb-4f9b-a6e4-9cdbb179d91e.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/abb204b0-7f89-4d97-b4bb-763db11488fb.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/acfcd2c5-8256-4408-9e53-4032be899756.json
/tmp/aegis-stress-tkp5It/proj/.aegis/runs/b00302d2-ce45-4601-97f9-a3e48ca32179.json
/tmp/aegis-stress-tkp5It/proj/.aegis/MEMORY.md

metrics: {
  "run_count": 28,
  "heal_attempts": 5,
  "heal_successes": 2,
  "last_run_id": "58020678-f253-4a4f-a4ba-0e5e88f9e6cd",
  "last_run_at": "2026-07-12T05:47:12.336556894+00:00"
}
```

## Log

`/tmp/aegis-stress-tkp5It/stress.log`

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

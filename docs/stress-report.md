# Aegis Stress Report

- Date: 2026-07-12T00:40:22-05:00
- Version: `aegis 0.8.0`
- Duration: 252s (~4.2 min)
- STRESS_ROOT: `/tmp/aegis-stress-SSl0Gk`
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
/tmp/aegis-stress-SSl0Gk/proj/Cargo.toml
/tmp/aegis-stress-SSl0Gk/proj/README.md
/tmp/aegis-stress-SSl0Gk/proj/Cargo.lock
/tmp/aegis-stress-SSl0Gk/proj/src/lib.rs
/tmp/aegis-stress-SSl0Gk/proj/src/lib.rs.bak
/tmp/aegis-stress-SSl0Gk/proj/src/math.rs
/tmp/aegis-stress-SSl0Gk/proj/.github/workflows/aegis-qa.yml
/tmp/aegis-stress-SSl0Gk/proj/docs/C_A.md
/tmp/aegis-stress-SSl0Gk/proj/docs/NOTE2.md
/tmp/aegis-stress-SSl0Gk/proj/docs/C_C.md
/tmp/aegis-stress-SSl0Gk/proj/docs/LOCK.txt
/tmp/aegis-stress-SSl0Gk/proj/docs/C_B.md
/tmp/aegis-stress-SSl0Gk/proj/docs/wiki/Architecture.md
/tmp/aegis-stress-SSl0Gk/proj/docs/wiki/Commands.md
/tmp/aegis-stress-SSl0Gk/proj/docs/wiki/Modules.md
/tmp/aegis-stress-SSl0Gk/proj/docs/wiki/Conventions.md
/tmp/aegis-stress-SSl0Gk/proj/docs/wiki/Home.md
/tmp/aegis-stress-SSl0Gk/proj/docs/wiki/_Sidebar.md
/tmp/aegis-stress-SSl0Gk/proj/docs/NOTE1.md
/tmp/aegis-stress-SSl0Gk/proj/docs/NOTE3.md
/tmp/aegis-stress-SSl0Gk/proj/.aegis/reviews/diff-20260712_053843.md
/tmp/aegis-stress-SSl0Gk/proj/.aegis/LESSONS.jsonl
/tmp/aegis-stress-SSl0Gk/proj/.aegis/dreams/2026-07-12_053832.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/dreams/2026-07-12_053832.md
/tmp/aegis-stress-SSl0Gk/proj/.aegis/SKILLS/rust-lib-trailing-junk-heal.md
/tmp/aegis-stress-SSl0Gk/proj/.aegis/SKILLS/docs-note-batch-pattern.md
/tmp/aegis-stress-SSl0Gk/proj/.aegis/checkpoints/20260712_053905.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/checkpoints/20260712_053809.status.txt
/tmp/aegis-stress-SSl0Gk/proj/.aegis/checkpoints/20260712_053809.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/checkpoints/20260712_053905.status.txt
/tmp/aegis-stress-SSl0Gk/proj/.aegis/nexus/neural-summary.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/metrics.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/FAILURES.jsonl
/tmp/aegis-stress-SSl0Gk/proj/.aegis/readiness/report.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/4cfd1c83-4d6a-42cc-b74d-eab7f7a33664.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/2692081e-8e47-48c0-9aa9-0d0a3d6b6411.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/34fcee6f-169a-4674-8f57-e36d583aae73.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/5fa75056-c120-4c1f-81fc-fdff51c735e9.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/6de52d1f-e890-4b50-b07f-e50be813e014.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/44ac44d2-4957-4999-ade4-1c9795512e74.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/c7d4a490-5718-44c5-b6a6-358e0025c350.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/4ca73ce5-b4d8-43d9-bd58-6f8c2ecd43e1.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/74724045-6b74-4a25-8f21-222ea606c2bb.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/2375c603-b45d-450d-b204-63a826ac5344.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/9fe72089-2acd-4e91-97d5-d15f9e0b97c5.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/5a9baeb0-e622-4895-80ae-858c2ac29e82.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/c3e21b5f-a0bc-45b9-822a-7c3b09e126d9.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/706a979a-0ffd-486a-9882-ad6a731e06f4.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/455a3b9f-fec8-4b41-b005-8edbd8fa9b0d.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/96272257-b922-4202-9cb7-6e98cbe04045.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/79828e1d-16d6-42e9-b268-74eba3671413.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/8284fb3f-6ded-404e-a999-0ebfe8989287.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/runs/d23a0b5b-ed04-4dd3-a750-0226c606703a.json
/tmp/aegis-stress-SSl0Gk/proj/.aegis/MEMORY.md

metrics: {
  "run_count": 28,
  "heal_attempts": 5,
  "heal_successes": 2,
  "last_run_id": "4cfd1c83-4d6a-42cc-b74d-eab7f7a33664",
  "last_run_at": "2026-07-12T05:40:14.838456827+00:00"
}
```

## Log

`/tmp/aegis-stress-SSl0Gk/stress.log`

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

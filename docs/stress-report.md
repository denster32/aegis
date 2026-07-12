# Aegis Stress Report

- Date: 2026-07-11T23:10:44-05:00
- Duration: 231s
- STRESS_ROOT: `/tmp/aegis-stress-WV2f73`
- PASS: 15
- FAIL: 0
- SKIP: 0

## Phases

See log: `/tmp/aegis-stress-WV2f73/stress.log`

## Interpretation

- Failures in S3 (heal) or S10 (concurrency) are P0.
- S5–S9 depend on API quality; soft pass notes in log.

## Next

```bash
./scripts/stress_test.sh
```

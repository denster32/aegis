# Stress testing Aegis

```bash
./scripts/stress_test.sh
```

Requires Grok OAuth (`aegis auth status`). Unsets `XAI_API_KEY` to prefer subscription path.

## Phases

| ID | Scenario |
|----|----------|
| S0 | CLI / auth / readiness / factory |
| S1 | Cold create Rust crate + tests |
| S2 | Multi-step edit |
| S3 | Induced compile failure + self-heal |
| S4 | Multi-file writes |
| S5 | Mission |
| S6 | Dream |
| S7 | Wiki |
| S8 | QA |
| S9 | Review --diff |
| S10 | Concurrent agents |
| S12 | Short turn / no hang |

Results: [stress-report.md](./stress-report.md)
